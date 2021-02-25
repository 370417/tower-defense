import { Application, Container, Sprite, Texture } from 'pixi.js';
import { TILE_SIZE } from '../constants';
import { Mob, State } from '../state';
import { spawnSmokeTrail } from './missile-smoke';
import { createMob } from './move-mobs';

export function operateMissileTower(state: State, app: Application): void {
    const { missileSpawners, towers, missiles, mobs, sprites } = state.components;
    for (const [entity, spawner] of missileSpawners) {
        const tower = towers.get(entity);
        if (tower) {
            if (spawner.reloadCountdown <= 0) {
                const target = selectTarget(state);

                if (target === undefined) {
                    return;
                }

                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                const targetMob = mobs.get(target)!;

                // spawn missile

                const x = (tower.col + 0.5) * TILE_SIZE + 0.5;
                const y = (tower.row + 0.5) * TILE_SIZE + 0.5;

                const mob = createMob(x, y);
                mob.rotation = Math.atan2(targetMob.y - y, targetMob.x - x);

                mobs.set(state.nextEntity, mob);

                missiles.set(state.nextEntity, {
                    targetEntity: 0,
                    acceleration: 0.15,
                    speed: 0,
                    topSpeed: 7,
                    turnRadius: 3 * TILE_SIZE,
                });

                const sprite = missileSprite();
                sprites.set(state.nextEntity, sprite);
                app.stage.addChild(sprite);

                spawnSmokeTrail(state.nextEntity, x, y, state, app);

                state.nextEntity += 1;

                spawner.reloadCountdown = spawner.reloadCost;
            } else {
                spawner.reloadCountdown -= 1;
            }
        }
    }
}

function selectTarget(state: State): number | undefined {
    const { walkVelocity, mobs } = state.components;
    for (const entity of walkVelocity.keys()) {
        const mob = mobs.get(entity);
        if (mob) {
            return entity;
        }
    }
}

export function missileSprite(): Sprite {
    const missile = Sprite.from(Texture.WHITE);
    missile.width = 10;
    missile.height = 7;
    missile.anchor.x = 0;
    missile.anchor.y = 0.5;
    missile.tint = 0x000000;
    return missile;
}

export function updateMissile(state: State, app: Application): void {
    const { missiles, mobs, sprites } = state.components;
    for (const [entity, missile] of missiles) {
        const targetMob = mobs.get(missile.targetEntity);
        const mob = mobs.get(entity);
        if (mob && targetMob) {
            // Aim towards the target
            const newRotation = Math.atan2(targetMob.y - mob.y, targetMob.x - mob.x);

            let dRotation = (newRotation - mob.rotation) % (2 * Math.PI);
            if (dRotation < Math.PI) { dRotation += 2 * Math.PI; }
            if (dRotation > Math.PI) { dRotation -= 2 * Math.PI; }
            const maxTurnSpeed = turnSpeed(missile.topSpeed, missile.turnRadius);
            dRotation = Math.min(maxTurnSpeed, Math.max(-maxTurnSpeed, dRotation));

            mob.rotation += dRotation;

            // Accelerate and cap speed
            missile.speed += missile.acceleration;
            missile.speed *= friction(missile.acceleration, missile.topSpeed);
            mob.tempDx += missile.speed * Math.cos(mob.rotation);
            mob.tempDy += missile.speed * Math.sin(mob.rotation);

            const collided = checkCollision(mob, targetMob, 5);
            const sprite = sprites.get(entity);
            if (collided && sprite) {
                app.stage.removeChild(sprite);
                mobs.delete(entity);
                missiles.delete(entity);
                sprites.delete(entity);
            }
        } else if (mob) {
            // select a new target, and if none exist,
            // go back to the original tower and circle
        }
    }
}

function friction(acceleration: number, topSpeed: number): number {
    return topSpeed / (acceleration + topSpeed);
}

function turnSpeed(topSpeed: number, turnRadius: number) {
    return topSpeed / turnRadius;
}

function checkCollision(mob: Mob, targetMob: Mob, threshold: number) {
    const dx = targetMob.x - mob.x;
    const dy = targetMob.y - mob.y;
    const distanceSquared = dx * dx + dy * dy;
    return distanceSquared < threshold * threshold;
}
