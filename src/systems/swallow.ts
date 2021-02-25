import { Application, Sprite, Texture } from 'pixi.js';
import { TILE_SIZE } from '../constants';
import { State } from '../state';
import { createMob } from './move-mobs';
import { baseTowerSprite } from './render-towers';

export function createSwallowTower(state: State, app: Application, row: number, col: number): void {
    const towerSprite = baseTowerSprite();
    towerSprite.x = col * TILE_SIZE;
    towerSprite.y = row * TILE_SIZE;
    app.stage.addChild(towerSprite);

    const towerEntity = state.nextEntity;
    state.components.towers.set(state.nextEntity, { row, col });
    state.nextEntity += 1;

    const x = (col + 0.5) * TILE_SIZE + 0.5;
    const y = (row + 0.5) * TILE_SIZE + 0.5;

    state.components.swallows.set(state.nextEntity, {
        roost: towerEntity,
        targetEntity: towerEntity,
        fixedSpeed: 0,
        turnRadius: 0,
        vanishingX: x,
        vanishingY: y,
    });

    state.components.mobs.set(state.nextEntity, createMob(x, y));

    const swallowSprite = baseSwallowSprite();
    swallowSprite.x = x;
    swallowSprite.y = y;
    app.stage.addChild(swallowSprite);

    state.nextEntity += 1;
}

export function baseSwallowSprite(): Sprite {
    const swallow = Sprite.from(Texture.WHITE);
    swallow.width = 16;
    swallow.height = 16;
    swallow.anchor.x = 0.5;
    swallow.anchor.y = 0.5;
    swallow.tint = 0x666666;
    return swallow;
}

export function operateSwallow(state: State): void {
    const { mobs, swallows } = state.components;
    for (const [entity, swallow] of swallows) {
        if (swallow.roost !== undefined) {
            //
        }
    }
}
