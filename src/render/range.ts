/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any */

import { Container, Point, SimpleRope, Texture } from 'pixi.js';

const points: Point[] = [];

const resolution = 80;

for (let i = 0; i <= resolution; i++) {
    points.push(new Point(0, 0));
}

const rope = new SimpleRope(Texture.WHITE, points, 2 / 16);
rope.visible = false;
rope.tint = 0x000000;

export function renderRange(x: number, y: number, radius: number): void {
    rope.visible = true;
    rope.x = x;
    rope.y = y;
    for (let i = 0; i <= resolution; i++) {
        points[i].x = radius * Math.cos(2 * Math.PI * i / resolution);
        points[i].y = radius * Math.sin(2 * Math.PI * i / resolution);
    }
}

export function recycleRange(): void {
    rope.visible = false;
}

export function initRangeRendering(container: Container): void {
    container.addChild(rope);

    (window as any).render_range = renderRange;

    (window as any).recycle_range = recycleRange;
}
