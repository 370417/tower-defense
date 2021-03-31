/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any */

import { Container, Sprite, Texture } from 'pixi.js';
import { TILE_SIZE } from '../constants';

const swallows = new Map<number, Sprite>();
const swallowPool: Sprite[] = [];

export function initSwallowRendering(container: Container, texture: Texture): void {
    (window as any).create_swallow = function (id: number) {
        const swallow = swallowPool.pop();
        if (!swallow) {
            const swallow = Sprite.from(texture);

            swallow.width = TILE_SIZE * 0.8;
            swallow.height = TILE_SIZE * 0.8;

            swallow.anchor.x = 0.5;
            swallow.anchor.y = 0.5;

            swallow.tint = 0x000000;

            container.addChild(swallow);
            swallows.set(id, swallow);
        } else {
            swallow.visible = true;
            swallows.set(id, swallow);
        }
    };

    (window as any).render_swallow = function (id: number, x: number, y: number, rotation: number, fade: number) {
        const swallow = swallows.get(id);
        if (swallow) {
            swallow.alpha = 1 - fade;
            swallow.x = x;
            swallow.y = y;
            swallow.rotation = rotation;
        }
    };

    (window as any).recycle_swallow = function (id: number) {
        const swallow = swallows.get(id);
        if (swallow) {
            swallow.visible = false;
            swallows.delete(id);
            swallowPool.push(swallow);
        }
    };
}
