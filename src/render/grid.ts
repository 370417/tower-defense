/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any */

import { Container, Sprite, Texture } from 'pixi.js';
import { MAP_HEIGHT, MAP_WIDTH, TILE_SIZE } from '../constants';

export function drawGrid(container: Container): void {
    // Inner lines

    for (let row = 1; row < MAP_HEIGHT; row++) {
        const line = Sprite.from(Texture.WHITE);
        line.width = MAP_WIDTH * TILE_SIZE;
        line.height = 1;
        line.tint = 0xCCCCCC;
        if (row === 0 || row === MAP_HEIGHT) {
            line.tint = 0x888888;
        }
        line.x = 0;
        line.y = row * TILE_SIZE;
        container.addChild(line);
    }

    for (let col = 1; col < MAP_WIDTH; col++) {
        const line = Sprite.from(Texture.WHITE);
        line.width = 1;
        line.height = MAP_HEIGHT * TILE_SIZE;
        line.tint = 0xCCCCCC;
        if (col === 0 || col === MAP_WIDTH) {
            line.tint = 0x888888;
        }
        line.x = col * TILE_SIZE;
        line.y = 0;
        container.addChild(line);
    }

    // Outer lines

    const lineNorth = Sprite.from(Texture.WHITE);
    lineNorth.width = MAP_WIDTH * TILE_SIZE + 1;
    lineNorth.height = 1;
    lineNorth.tint = 0x888888;
    lineNorth.x = 0;
    lineNorth.y = 0;
    container.addChild(lineNorth);

    const lineSouth = Sprite.from(Texture.WHITE);
    lineSouth.width = MAP_WIDTH * TILE_SIZE + 1;
    lineSouth.height = 1;
    lineSouth.tint = 0x888888;
    lineSouth.x = 0;
    lineSouth.y = MAP_HEIGHT * TILE_SIZE;
    container.addChild(lineSouth);

    const lineEast = Sprite.from(Texture.WHITE);
    lineEast.width = 1;
    lineEast.height = MAP_HEIGHT * TILE_SIZE + 1;
    lineEast.tint = 0x888888;
    lineEast.x = 0;
    lineEast.y = 0;
    container.addChild(lineEast);

    const lineWest = Sprite.from(Texture.WHITE);
    lineWest.width = 1;
    lineWest.height = MAP_HEIGHT * TILE_SIZE + 1;
    lineWest.tint = 0x888888;
    lineWest.x = MAP_WIDTH * TILE_SIZE;
    lineWest.y = 0;
    container.addChild(lineWest);

}

export function initPathRendering(container: Container): void {
    (window as any).render_path_tile = function (row: number, col: number): void {
        const square = Sprite.from(Texture.WHITE);
        square.tint = 0xE8E8E8;
        square.width = TILE_SIZE + 1;
        square.height = TILE_SIZE + 1;
        square.x = col * TILE_SIZE;
        square.y = row * TILE_SIZE;
        container.addChild(square);
    };

    (window as any).render_path_border = function (row: number, col: number, horizontal: boolean): void {
        if (horizontal) {
            const edge = Sprite.from(Texture.WHITE);
            edge.tint = 0x000000;
            edge.width = TILE_SIZE + 1;
            edge.height = 1;
            edge.x = col * TILE_SIZE;
            edge.y = row * TILE_SIZE;
            container.addChild(edge);
        } else {
            const edge = Sprite.from(Texture.WHITE);
            edge.tint = 0x000000;
            edge.width = 1;
            edge.height = TILE_SIZE + 1;
            edge.x = col * TILE_SIZE;
            edge.y = row * TILE_SIZE;
            container.addChild(edge);
        }
    };
}
