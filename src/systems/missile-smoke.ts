import { Application, Point, SimpleRope, Texture } from 'pixi.js';
import { State } from '../state';

const smokeSpacing = 1;
const smokeBufferLen = 40;

const smokeTexture = createSmokeTexture();

export function spawnSmokeParticles(state: State, app: Application): void {
    const { smokeParticles, smokeTrails, missiles, mobs } = state.components;
    for (const [entity, smokeTrail] of smokeTrails) {
        const missile = missiles.get(smokeTrail.missileEntity);
        const missileMob = mobs.get(smokeTrail.missileEntity);

        if (!missile || !missileMob) {
            // remove the smokeTrail entirely, at least for now
            app.stage.removeChild(smokeTrail.rope);
            // don't call rope.destroy() -- is that okay?
            smokeTrails.delete(entity);
            return;
        }

        if (smokeTrail.age % smokeSpacing === 0) {
            smokeParticles.set(state.nextEntity, {
                age: 0,
                smokeTrailEntity: entity,
                x: missileMob.x,
                y: missileMob.y,
                // normalX: -missileMob.tempDy / missile.speed,
                // normalY: missileMob.tempDx / missile.speed,
                normalX: -Math.sin(missileMob.rotation),
                normalY: Math.cos(missileMob.rotation),
            });
            state.nextEntity += 1;
        }

        smokeTrail.age += 1;
    }
}

export function updateSmokeParticles(state: State): void {
    const { smokeParticles } = state.components;
    for (const [entity, smokeParticle] of smokeParticles) {
        if (smokeParticle.age >= smokeSpacing * smokeBufferLen) {
            smokeParticles.delete(entity);
        }
        smokeParticle.age += 1;
    }
}

export function renderSmoke(state: State): void {
    const { smokeParticles, smokeTrails } = state.components;

    for (const smokeParticle of smokeParticles.values()) {
        const smokeTrail = smokeTrails.get(smokeParticle.smokeTrailEntity);
        if (smokeTrail) {
            // If less than a full trail has been generated, we want to adjust
            // particle ages to appear as old as possible. That way, smoke
            // trails start out more faded rather than more visible.

            const i = Math.floor(smokeParticle.age / smokeSpacing);
            if (i < smokeBufferLen) {
                const birthtick = state.tick - smokeParticle.age;
                smokeTrail.points[i].x = smokeParticle.x + (i + 3) * Math.sin(smokeTrail.frequency * birthtick + smokeTrail.shift) * smokeParticle.normalX;
                smokeTrail.points[i].y = smokeParticle.y + (i + 3) * Math.sin(smokeTrail.frequency * birthtick + smokeTrail.shift) * smokeParticle.normalY;
            }
        }
    }
}

export function spawnSmokeTrail(missileEntity: number, towerX: number, towerY: number, state: State, app: Application): void {
    const { smokeTrails } = state.components;
    const frequencies = [0.2, 0.12, 0.2];
    const shifts = [0, 2 / 3 * Math.PI, 4 / 3 * Math.PI];
    for (let i = 0; i < 3; i++) {
        const points: Point[] = [];
        for (let i = 0; i < smokeBufferLen; i++) {
            points.push(new Point(towerX, towerY));
        }

        const rope = new SimpleRope(smokeTexture, points);
        rope.tint = 0xBBBBBB;
        app.stage.addChild(rope);

        smokeTrails.set(state.nextEntity, {
            age: 0,
            missileEntity,
            points,
            rope,
            frequency: frequencies[i],
            shift: shifts[i],
        });
        state.nextEntity += 1;
    }
}

function createSmokeTexture(): Texture {
    const canvas = document.createElement('canvas');
    canvas.width = 128;
    canvas.height = 2;
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const ctx = canvas.getContext('2d')!;

    const gradient = ctx.createLinearGradient(0, 0, canvas.width, 0);
    gradient.addColorStop(0, '#FFF');
    gradient.addColorStop(1, '#FFF0');

    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    return Texture.fromBuffer(new Uint8Array(ctx.getImageData(0, 0, canvas.width, canvas.height).data), canvas.width, canvas.height);
}
