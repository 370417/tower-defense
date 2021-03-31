/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any */

import { Container, Sprite, Texture } from 'pixi.js';
import { TILE_SIZE } from '../constants';

const indicators = new Map<number, Sprite>();
const indicatorPool: Sprite[] = [];

export function initIndicatorRendering(container: Container, texture: Texture): void {
    (window as any).create_indicator = function (id: number) {
        const indicator = indicatorPool.pop();
        if (!indicator) {
            const indicator = Sprite.from(texture);

            indicator.width = TILE_SIZE * 0.5;
            indicator.height = TILE_SIZE * 0.5;

            indicator.anchor.x = 0.5;
            indicator.anchor.y = 0.5;

            container.addChild(indicator);
            indicators.set(id, indicator);
        } else {
            indicator.visible = true;
            indicators.set(id, indicator);
        }
    };

    (window as any).render_indicator = function (id: number, x: number, y: number) {
        const indicator = indicators.get(id);
        if (indicator) {
            indicator.x = x;
            indicator.y = y - 0.65 * TILE_SIZE;
        }
    };

    (window as any).recycle_indicator = function (id: number) {
        const indicator = indicators.get(id);
        if (indicator) {
            indicator.visible = false;
            indicators.delete(id);
            indicatorPool.push(indicator);
        }
    };
}
