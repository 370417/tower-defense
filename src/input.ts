import { TILE_SIZE } from './constants';
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

export function bufferInput(input: Input): void {
    localInputBuffer[0].push(input);
}

export const nonSyncInputBuffer: NonSyncInput[] = [];

export const inputAvailable = true;

type Input = {
    type: 'build tower',
    row: number,
    col: number,
    towerIndex: number,
} | {
    type: 'skip back',
} | {
    type: 'play pause',
} | {
    type: 'fast forward',
} | {
    type: 'send next wave'
} | {
    type: 'cancel tower',
    row: number,
    col: number,
};

// NonSyncInputs do not affect core game state, so they do not need to be
// synchronized across the network.
type NonSyncInput = {
    type: 'select tower by pos',
    row: number,
    col: number,
};

// Instead of calling methods on world directly, we store events and pass them
// to wasm in the game loop. This avoids seemingly 'copying' the &mut World.
// For the sake of correctness, we capture all clicks that happen in a frame,
// but for hovering, we only care about the mouse's final position.

export let mouseRow = -1;
export let mouseCol = -1;

export function initGridInput(canvas: HTMLElement): void {
    canvas.addEventListener('mouseleave', () => {
        mouseRow = -1;
        mouseCol = -1;
    });

    canvas.addEventListener('mousemove', event => {
        const row = Math.floor(event.offsetY / TILE_SIZE);
        const col = Math.floor(event.offsetX / TILE_SIZE);
        if (row !== mouseRow || col !== mouseCol) {
            mouseRow = row;
            mouseCol = col;
        }
    });

    canvas.addEventListener('click', event => {
        const row = Math.floor(event.offsetY / TILE_SIZE);
        const col = Math.floor(event.offsetX / TILE_SIZE);
        if (clickedTower?.towerStatus === 'prototype') {
            // Try and build a new tower
            bufferInput({
                type: 'build tower',
                row,
                col,
                towerIndex: clickedTower.towerIndex,
            });
        } else {
            // Try and select a tower
            nonSyncInputBuffer.push({
                type: 'select tower by pos',
                row,
                col,
            });
        }
    });

    canvas.addEventListener('contextmenu', event => {
        const row = Math.floor(event.offsetY / TILE_SIZE);
        const col = Math.floor(event.offsetX / TILE_SIZE);
        if (clickedTower?.towerStatus === 'prototype') {
            cancelTowerSelect();
        }
        bufferInput({
            type: 'cancel tower',
            row,
            col,
        });
        event.preventDefault();
    });
}
