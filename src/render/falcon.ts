/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any */

import { Container, Sprite, Texture } from 'pixi.js';
import { TILE_SIZE } from '../constants';

const falcons = new Map<number, Sprite>();
const falconPool: Sprite[] = [];

export function initFalconRendering(container: Container, texture: Texture): void {
    (window as any).create_falcon = function (id: number) {
        const falcon = falconPool.pop();
        if (!falcon) {
            const falcon = Sprite.from(texture);

            falcon.width = TILE_SIZE * 0.75;
            falcon.height = TILE_SIZE * 0.75;

            falcon.anchor.x = 0.5;
            falcon.anchor.y = 0.5;

            falcon.tint = 0x000000;

            container.addChild(falcon);
            falcons.set(id, falcon);
        } else {
            falcon.visible = true;
            falcons.set(id, falcon);
        }
    };

    (window as any).render_falcon = function (id: number, x: number, y: number, rotation: number, fade: number) {
        const falcon = falcons.get(id);
        if (falcon) {
            falcon.alpha = 1 - fade;
            falcon.x = x;
            falcon.y = y;
            falcon.rotation = rotation;
        }
    };

    (window as any).recycle_falcon = function (id: number) {
        const falcon = falcons.get(id);
        if (falcon) {
            falcon.visible = false;
            falcons.delete(id);
            falconPool.push(falcon);
        }
    };
}
