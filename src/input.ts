import { TILE_SIZE } from './constants';
import { recycleRange } from './render/range';
import { cancelTowerSelect, clickedTower } from './tower-select';

const tickDelay = 7;

// We are using deterministic lockstep (delay-based netcode, in fighting game
// terms). We store the latest [tickDelay + 1] ticks of input in this buffer.
// Since it is small, we just unshift and pop instead of using a ring buffer.
// So: inputs in the buffer are stored in order from newest to oldest.

// And note: we can send relevant non-game-affecting inputs (just map hovers,
// really) eagerly instead of buffering them.
export const localInputBuffer: Input[][] = [];
for (let i = 0; i < tickDelay + 1; i++) {
    localInputBuffer.push([]);
}

export const inputAvailable = true;

type Input = {
    type: 'build tower',
    row: number,
    col: number,
    towerIndex: number,
};

// Instead of calling methods on world directly, we store events and pass them
// to wasm in the game loop. This avoids seemingly 'copying' the &mut World.
// For the sake of correctness, we capture all clicks that happen in a frame,
// but for hovering, we only care about the mouse's final position.

type Pos = {
    row: number,
    col: number,
};

export function initGridInput(canvas: HTMLElement, mousePos: Pos): void {
    canvas.addEventListener('mouseleave', () => {
        recycleRange();
        mousePos.row = -1;
        mousePos.col = -1;
    });

    canvas.addEventListener('mousemove', event => {
        const row = Math.floor(event.offsetY / TILE_SIZE);
        const col = Math.floor(event.offsetX / TILE_SIZE);
        if (row !== mousePos.row || col !== mousePos.col) {
            mousePos.row = row;
            mousePos.col = col;
        }
    });

    canvas.addEventListener('click', event => {
        const row = Math.floor(event.offsetY / TILE_SIZE);
        const col = Math.floor(event.offsetX / TILE_SIZE);
        if (clickedTower?.towerStatus === 'prototype') {
            localInputBuffer[0].push({
                type: 'build tower',
                row,
                col,
                towerIndex: clickedTower.towerIndex,
            });
        }
    });

    canvas.addEventListener('contextmenu', event => {
        const row = Math.floor(event.offsetY / TILE_SIZE);
        const col = Math.floor(event.offsetX / TILE_SIZE);
        if (clickedTower?.towerStatus === 'prototype') {
            event.preventDefault();
            cancelTowerSelect();
        }
    });
}
