import { ParticleContainer, Sprite, Texture } from 'pixi.js';
import { TILE_SIZE } from '../constants';

// export function renderTowers({ towers, sprites }: Components): void {
//     for (const [entity, { row, col }] of towers) {
//         const sprite = sprites.get(entity);
//         if (sprite) {
//             sprite.x = 
//         }
//     }
// }

export function baseTowerSprite(): ParticleContainer {
    const tower = new ParticleContainer(10);

    const border = Sprite.from(Texture.WHITE);
    border.tint = 0x000000;
    border.width = TILE_SIZE + 1;
    border.height = TILE_SIZE + 1;
    tower.addChild(border);

    const background = Sprite.from(Texture.WHITE);
    background.tint = 0xF8F8F8;
    background.width = TILE_SIZE - 1;
    background.height = TILE_SIZE - 1;
    background.x = 1;
    background.y = 1;
    tower.addChild(background);

    return tower;
}
