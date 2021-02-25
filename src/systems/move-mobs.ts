// The mob component stores acceleration, but this values are
// temporary, for the current tick only. They exist so that multiple sources
// of acceleration can mix without interfering with each other's
// sources of truth. The moveMobs system processes this temporary values
// and resets it to 0.

import { Components, Mob } from '../state';

export function moveMobs({ mobs }: Components): void {
    for (const mob of mobs.values()) {
        mob.rotation += mob.dRotation;

        mob.dx += mob.tempDdx;
        mob.dy += mob.tempDdy;

        mob.x += mob.dx + mob.tempDx;
        mob.y += mob.dy + mob.tempDy;

        mob.tempDdx = 0;
        mob.tempDdy = 0;
        mob.tempDx = 0;
        mob.tempDy = 0;
    }
}

export function createMob(x: number, y: number): Mob {
    return {
        x,
        y,
        rotation: 0,
        dRotation: 0,
        dx: 0,
        dy: 0,
        tempDx: 0,
        tempDy: 0,
        tempDdx: 0,
        tempDdy: 0,
    };
}
