import { TILE_SIZE } from './constants';
import { recycleRange } from './render/range';

// Instead of calling methods on world directly, we store events and pass them
// to wasm in the game loop. This avoids seemingly 'copying' the &mut World.
// For the sake of correctness, we capture all clicks that happen in a frame,
// but for hovering, we only care about the mouse's final position.

type Pos = {
    row: number,
    col: number,
};

export function initGridInput(canvas: HTMLElement, mousePos: Pos, mouseClicks: Pos[]): void {
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
}
