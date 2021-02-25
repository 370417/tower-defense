import { Components } from '../state';

export function renderMobPositions({ mobs, sprites }: Components, timeTillRender: number): void {
    for (const [entity, { x, y, dx, dy, rotation, dRotation }] of mobs) {
        const sprite = sprites.get(entity);
        if (sprite) {
            // Interpolate position if we are rendering between frames
            sprite.x = x + timeTillRender * dx;
            sprite.y = y + timeTillRender * dy;

            sprite.rotation = rotation + timeTillRender * dRotation;
        }
    }
}
