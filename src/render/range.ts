/* eslint-disable @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any */

import { Container, Graphics } from 'pixi.js';
import { TILE_SIZE } from '../constants';
import { mouseCol, mouseRow } from '../input';

const rangeGraphic = new Graphics();
rangeGraphic.visible = false;
export function initRangeRendering(container: Container): void {
    container.addChild(rangeGraphic);
}


// When we have selected a tower to build but haven't placed it down yet, we
// still want to display a range. There is no actual tower, so we need to
// get the range separately. Putting it in a different variable prevents it
// from being overridden.
let previewTowerRadius = 0;
let canBuildTower = true;
export function setPreviewTowerInfo(radius: number, canBuild: boolean): void {
    previewTowerRadius = radius;
    canBuildTower = canBuild;
}

let currRadius = 0;
export function renderRange(x: number, y: number, radius: number): void {
    if (previewTowerRadius > 0) {
        currRadius = radius;
        const x = (mouseCol + 0.5) * TILE_SIZE;
        const y = (mouseRow + 0.5) * TILE_SIZE;
        renderGraphic(previewTowerRadius, x, y, canBuildTower ? 0x000000 : 0xff0000);
        rangeGraphic.visible = true;
    } else if (radius === 0) {
        currRadius = radius;
        rangeGraphic.visible = false;
        rangeGraphic.clear();
    } else if (radius !== currRadius) {
        currRadius = radius;
        renderGraphic(radius, x, y, 0x000000);
        rangeGraphic.visible = true;
    } else {
        renderGraphic(radius, x, y, 0x000000);
        rangeGraphic.visible = true;
    }
}

// It might feel wasteful to rerender when x and y change but radius does not
// change. Why not just move the graphic around?
// Moving graphics around appears to be non-instantaneous and non-blocking, so
// that more efficient solution caused ranges to flash in their old location
// before becoming correct. Rerendering means that we clear the graphic
// between non-contiguous towers, so there is no visible flashing.
function renderGraphic(radius: number, x: number, y: number, color: number) {
    rangeGraphic.clear();
    rangeGraphic.beginFill(color, 0.125);
    rangeGraphic.drawCircle(x, y, radius);
    rangeGraphic.beginHole();
    rangeGraphic.drawRect(-TILE_SIZE / 2, -TILE_SIZE / 2, TILE_SIZE, TILE_SIZE);
    rangeGraphic.endHole();
    rangeGraphic.endFill();

    // Draw the border of the circle separately.
    // We do this because the filled circle has a square hole where the
    // tower would be, and that hole gets a line drawn around it as well.
    // And the line around the hole doesn't align with the tower perfectly.
    // So this separate circle avoids the line around the hole.
    rangeGraphic.lineStyle(1, 0x000000);
    rangeGraphic.drawCircle(x, y, radius);
}
