import { Graphics } from 'pixi.js';
import { TILE_SIZE } from '../constants';

export function renderProgress(graphics: Graphics, progress: number): void {
    const radius = TILE_SIZE / 2;
    const theta = 3 * Math.PI / 2 + progress * 2 * Math.PI;
    const rotationFromDiagonal = Math.abs((theta % (Math.PI / 2)) - Math.PI / 4);
    const rotationFromAxis = Math.PI / 4 - rotationFromDiagonal;
    const radiusScale = 1 / Math.cos(rotationFromAxis);

    const x = radius * radiusScale * Math.cos(theta);
    const y = radius * radiusScale * Math.sin(theta);

    graphics.clear();

    if (progress >= 1) {
        return;
    }

    graphics.lineStyle(0);
    graphics.beginFill(0x000000, 1);
    graphics.moveTo(0, -radius);
    graphics.lineTo(0, 0);
    graphics.lineTo(x, y);
    if (progress < 1 / 8) {
        graphics.lineTo(radius, -radius);
    }
    if (progress < 3 / 8) {
        graphics.lineTo(radius, radius);
    }
    if (progress < 5 / 8) {
        graphics.lineTo(-radius, radius);
    }
    if (progress < 7 / 8) {
        graphics.lineTo(-radius, -radius);
    }
    graphics.lineTo(0, -radius);
    graphics.closePath();
    graphics.endFill();
}
