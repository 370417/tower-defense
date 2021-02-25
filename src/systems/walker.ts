import { TILE_SIZE } from '../constants';
import { Components, State } from '../state';

// A mob with a walkVelocity component can move intentionally under it's own power.
// It moves by setting velocity, not acceleration.

// The walkVelocity component is separate from the mob's main velocity so that
// some systems can affect walking only. For example, electric-based paralysis
// should not prevent being pushed by an explosion, but should prevent running
// from a falcon.

export function planWalk({ walkVelocity, mobs }: Components, path: string[]): void {
    for (const [entity, velocity] of walkVelocity) {
        const mob = mobs.get(entity);
        if (mob) {
            const newVelocity = pathDirection(mob.x, mob.y, path);
            velocity.dx = newVelocity.dx;
            velocity.dy = newVelocity.dy;
        }
    }
}

export function executeWalk({ walkVelocity, mobs }: Components): void {
    for (const [entity, velocity] of walkVelocity) {
        const mob = mobs.get(entity);
        if (mob) {
            mob.tempDx += 1.5 * velocity.dx;
            mob.tempDy += 1.5 * velocity.dy;
        }
    }
}

export function loopWalkers({ walkVelocity, mobs }: Components, state: State): void {
    for (const entity of walkVelocity.keys()) {
        const mob = mobs.get(entity);
        if (mob) {
            const trueRow = 2 + Math.floor(mob.y / TILE_SIZE);
            const trueCol = 2 + Math.floor(mob.x / TILE_SIZE);
            for (let i = 0; i < state.map.exits.length; i++) {
                const exit = state.map.exits[i];
                if (trueRow === exit.row && trueCol === exit.col) {
                    const entrance = state.map.entrances[i] || state.map.entrances[0];
                    mob.x = (entrance.col - 1.5) * TILE_SIZE;
                    mob.y = (entrance.row - 1.5) * TILE_SIZE;
                }
            }
        }
    }
}

// x and y are screen coordinates in pixels,
// so they can be negative for the sections of the path
// that extend offscreen.
function pathDirection(x: number, y: number, path: string[]): {
    dx: number; dy: number;
} {
    const trueRow = Math.floor(y / TILE_SIZE) + 2;
    const trueCol = Math.floor(x / TILE_SIZE) + 2;

    switch (path[trueRow][trueCol]) {
        case '>':
            return { dx: 1, dy: 0 };
        case '<':
            return { dx: -1, dy: 0 };
        case 'n':
            return { dx: 0, dy: -1 };
        case 'v':
            return { dx: 0, dy: 1 };
    }

    // Assume we are at a turn
    // Find the entrance and exit to the turn

    let entranceX: number, entranceY: number;
    let entranceDirection: { dx: number; dy: number; };
    if (path[trueRow][trueCol - 1] === '>') {
        entranceX = 0;
        entranceY = TILE_SIZE / 2;
        entranceDirection = { dx: 1, dy: 0 };
    } else if (path[trueRow][trueCol + 1] === '<') {
        entranceX = TILE_SIZE;
        entranceY = TILE_SIZE / 2;
        entranceDirection = { dx: -1, dy: 0 };
    } else if (path[trueRow + 1][trueCol] === 'n') {
        entranceX = TILE_SIZE / 2;
        entranceY = TILE_SIZE;
        entranceDirection = { dx: 0, dy: -1 };
    } else {
        entranceX = TILE_SIZE / 2;
        entranceY = 0;
        entranceDirection = { dx: 0, dy: 1 };
    }

    let exitX: number, exitY: number;
    let exitDirection: { dx: number; dy: number; };
    if (path[trueRow][trueCol - 1] === '<') {
        exitX = 0;
        exitY = TILE_SIZE / 2;
        exitDirection = { dx: -1, dy: 0 };
    } else if (path[trueRow][trueCol + 1] === '>') {
        exitX = TILE_SIZE;
        exitY = TILE_SIZE / 2;
        exitDirection = { dx: 1, dy: 0 };
    } else if (path[trueRow + 1][trueCol] === 'v') {
        exitX = TILE_SIZE / 2;
        exitY = TILE_SIZE;
        exitDirection = { dx: 0, dy: 1 };
    } else {
        exitX = TILE_SIZE / 2;
        exitY = 0;
        exitDirection = { dx: 0, dy: -1 };
    }

    const localX = ((x % TILE_SIZE) + TILE_SIZE) % TILE_SIZE;
    const localY = ((y % TILE_SIZE) + TILE_SIZE) % TILE_SIZE;

    const distFromEntrace = Math.abs(localX - entranceX) + Math.abs(localY - entranceY);
    const distFromExit = Math.abs(localX - exitX) + Math.abs(localY - exitY);

    if (distFromEntrace < distFromExit) {
        return entranceDirection;
    } else {
        return exitDirection;
    }
}
