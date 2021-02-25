import { Container, Point, SimpleRope } from 'pixi.js';

export type State = {
    tick: number,
    map: {
        path: string[];
        entrances: { row: number; col: number; }[];
        exits: { row: number; col: number; }[];
    };
    components: Components;
    nextEntity: number;
};

export type Components = {
    mobs: Map<number, Mob>;
    sprites: Map<number, Container>;
    walkVelocity: Map<number, Velocity>;
    towers: Map<number, Tower>;
    missiles: Map<number, Missile>;
    missileSpawners: Map<number, MissileSpawner>;
    smokeParticles: Map<number, SmokeParticle>;
    smokeTrails: Map<number, SmokeTrail>;
    swallows: Map<number, Swallow>,
};

// Movable object
export type Mob = {
    x: number;
    y: number;
    rotation: number;
    dRotation: number;
    dx: number;
    dy: number;
    tempDx: number;
    tempDy: number;
    tempDdx: number;
    tempDdy: number;
};

export type Velocity = {
    dx: number;
    dy: number;
};

export type Tower = {
    row: number;
    col: number;
};

export type Missile = {
    targetEntity: number;
    acceleration: number;
    // We could derive speed from the mob component, but this is more convenient
    speed: number;
    topSpeed: number;
    turnRadius: number;
};

export type MissileSpawner = {
    reloadCountdown: number;
    reloadCost: number;
};

export type SmokeParticle = {
    age: number;
    smokeTrailEntity: number;
    x: number,
    y: number,
    normalX: number,
    normalY: number,
};

export type SmokeTrail = {
    age: number,
    missileEntity: number,
    points: Point[];
    rope: SimpleRope;
    frequency: number;
    shift: number;
};

export type Swallow = {
    // Store the entity that the swallow is in if it isn't out flying
    roost: number | undefined;
    // If the swallow is at roost, targetEntity === roost
    targetEntity: number,
    fixedSpeed: number;
    turnRadius: number;
    vanishingX: number;
    vanishingY: number;
};
