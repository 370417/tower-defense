import { Application, Container, Graphics, ParticleContainer, Point, SimpleRope, Sprite, Texture } from 'pixi.js';
import { MAP_WIDTH, TILE_SIZE, MAP_HEIGHT, MS_PER_UPDATE, MAX_UPDATES_PER_TICK } from './constants';

const loadBackend = import('../dist/tower_defense');
const loadMemory = import('../dist/tower_defense_bg.wasm');

const app = new Application({
    width: MAP_WIDTH * TILE_SIZE + 1,
    height: MAP_HEIGHT * TILE_SIZE + 1,
    backgroundColor: 0xFFFFFF,
    antialias: true,
});

document.body.appendChild(app.view);

app.loader.load((loader, resources) => {

    Promise.all([loadBackend, loadMemory]).then(([worldModule, memModule]) => {
        drawGrid();

        // Some graphics functions read wasm memory, so we bind those parameters

        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
        (window as any).render_smoke_trail = render_smoke_trail.bind(undefined, memModule.memory);

        const world = worldModule.World.new();

        // game loop with fixed time step, variable rendering */
        let lastUpdateTime = window.performance.now();
        let lag = 0;

        function gameTick() {
            const time = window.performance.now();
            const elapsed = time - lastUpdateTime;
            lastUpdateTime = time;
            lag += elapsed;

            if (lag > MS_PER_UPDATE * MAX_UPDATES_PER_TICK) {
                // Too much lag, just pretend it doesn't exist
                lag = 0;
                world.render(0);
                // Don't even process input. Is this a good idea?
                return;
            }

            // process input

            let updates = 0;
            while (lag >= MS_PER_UPDATE) {
                world.update();
                updates += 1;
                lag -= MS_PER_UPDATE;
                if (updates > MAX_UPDATES_PER_TICK) {
                    world.render(1);
                    return;
                }
            }

            world.render(lag / MS_PER_UPDATE);
        }

        app.ticker.add(gameTick);
    }).catch(console.error);

});

function drawGrid() {
    for (let row = 0; row <= MAP_HEIGHT; row++) {
        const line = Sprite.from(Texture.WHITE);
        line.width = MAP_WIDTH * TILE_SIZE;
        line.height = 1;
        line.tint = 0xCCCCCC;
        if (row === 0 || row === MAP_HEIGHT) {
            line.tint = 0x888888;
        }
        line.x = 0;
        line.y = row * TILE_SIZE;
        app.stage.addChild(line);
    }

    for (let col = 0; col <= MAP_WIDTH; col++) {
        const line = Sprite.from(Texture.WHITE);
        line.width = 1;
        line.height = MAP_HEIGHT * TILE_SIZE;
        line.tint = 0xCCCCCC;
        if (col === 0 || col === MAP_WIDTH) {
            line.tint = 0x888888;
        }
        line.x = col * TILE_SIZE;
        line.y = 0;
        app.stage.addChild(line);
    }
}

function render_path_tile(row: number, col: number): void {
    const square = Sprite.from(Texture.WHITE);
    square.tint = 0xE8E8E8;
    square.width = TILE_SIZE + 1;
    square.height = TILE_SIZE + 1;
    square.x = col * TILE_SIZE;
    square.y = row * TILE_SIZE;
    app.stage.addChild(square);
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).render_path_tile = render_path_tile;

function render_path_border(row: number, col: number, horizontal: boolean): void {
    if (horizontal) {
        const edge = Sprite.from(Texture.WHITE);
        edge.tint = 0x000000;
        edge.width = TILE_SIZE + 1;
        edge.height = 1;
        edge.x = col * TILE_SIZE;
        edge.y = row * TILE_SIZE;
        app.stage.addChild(edge);
    } else {
        const edge = Sprite.from(Texture.WHITE);
        edge.tint = 0x000000;
        edge.width = 1;
        edge.height = TILE_SIZE + 1;
        edge.x = col * TILE_SIZE;
        edge.y = row * TILE_SIZE;
        app.stage.addChild(edge);
    }
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).render_path_border = render_path_border;


const graphics = new Map<number, Container>();

function create_mob(id: number) {
    const mob = new Graphics();
    mob.beginFill(0x666666);
    mob.drawCircle(0, 0, 0.3 * TILE_SIZE);
    app.stage.addChild(mob);
    graphics.set(id, mob);
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).create_mob = create_mob;

function render_mob_position(id: number, x: number, y: number) {
    const mob = graphics.get(id);
    if (mob) {
        mob.x = x;
        mob.y = y;
    }
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).render_mob_position = render_mob_position;

function create_tower(id: number, row: number, col: number) {
    const tower = new ParticleContainer(10);
    tower.x = col * TILE_SIZE;
    tower.y = row * TILE_SIZE;

    const border = Sprite.from(Texture.WHITE);
    border.tint = 0x000000;
    border.width = TILE_SIZE + 1;
    border.height = TILE_SIZE + 1;
    tower.addChild(border);

    const background = Sprite.from(Texture.WHITE);
    background.tint = 0xF8F8F8;
    background.width = TILE_SIZE - 1;
    background.height = TILE_SIZE - 1;
    background.x = 1;
    background.y = 1;
    tower.addChild(background);

    app.stage.addChild(tower);
    graphics.set(id, tower);
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).create_tower = create_tower;

const missilePool: Sprite[] = [];

function create_missile(id: number) {
    const missile = missilePool.pop() || Sprite.from(Texture.WHITE);
    missile.visible = true;
    missile.tint = 0x000000;
    missile.width = 10;
    missile.height = 7;
    missile.anchor.x = 0;
    missile.anchor.y = 0.5;
    app.stage.addChild(missile);
    graphics.set(id, missile);
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).create_missile = create_missile;

function render_missile(id: number, x: number, y: number, rotation: number) {
    const missile = graphics.get(id);
    if (missile) {
        missile.x = x;
        missile.y = y;
        missile.rotation = rotation;
    }
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).render_missile = render_missile;

function recycle_missile(id: number) {
    const missile = graphics.get(id);
    if (missile) {
        missile.visible = false;
        graphics.delete(id);
        missilePool.push(missile as unknown as Sprite);
    }
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).recycle_missile = recycle_missile;

type SmokeTrail = [SimpleRope, Point[]]
const smokeTrails = new Map<number, SmokeTrail>();
const smokeTrailPool: SmokeTrail[] = [];

const smokeTexture = createSmokeTexture();

function create_smoke_trail(id: number, maxLength: number) {
    const recycledSmokeTrail = smokeTrailPool.pop();
    if (recycledSmokeTrail) {
        recycledSmokeTrail[0].visible = true;
        smokeTrails.set(id, recycledSmokeTrail);
    } else {
        const points: Point[] = [];
        for (let i = 0; i < maxLength; i++) {
            points.push(new Point(0, 0));
        }
        const rope = new SimpleRope(smokeTexture, points);
        // rope.visible = true;
        rope.tint = 0xBBBBBB;
        smokeTrails.set(id, [rope, points]);
        app.stage.addChild(rope);
    }
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).create_smoke_trail = create_smoke_trail;

function render_smoke_trail(memory: WebAssembly.Memory, id: number, x_ptr: number, y_ptr: number) {
    const smokeTrail = smokeTrails.get(id);
    if (smokeTrail) {
        const points = smokeTrail[1];
        const xs = new Float32Array(memory.buffer, x_ptr, points.length);
        const ys = new Float32Array(memory.buffer, y_ptr, points.length);
        for (let i = 0; i < points.length; i++) {
            points[i].x = xs[i];
            points[i].y = ys[i];
        }
    }
}
// We need to bind the memory paramter, so we attach render_smoke_trail to
// window only after loading the wasm

function recycle_smoke_trail(id: number) {
    const smokeTrail = smokeTrails.get(id);
    if (smokeTrail) {
        smokeTrail[0].visible = false;
        smokeTrails.delete(id);
        smokeTrailPool.push(smokeTrail);
    }
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).recycle_smoke_trail = recycle_smoke_trail;

function createSmokeTexture(): Texture {
    const canvas = document.createElement('canvas');
    canvas.width = 128;
    canvas.height = 2;
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const ctx = canvas.getContext('2d')!;

    const gradient = ctx.createLinearGradient(0, 0, canvas.width, 0);
    gradient.addColorStop(0, '#FFF0');
    gradient.addColorStop(1, '#FFF');

    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    return Texture.fromBuffer(new Uint8Array(ctx.getImageData(0, 0, canvas.width, canvas.height).data), canvas.width, canvas.height);
}

type Explosion = [SimpleRope, Point[]];
const explosions = new Map<number, Explosion>();
const explosionPool: Explosion[] = [];

// const unitCircleX: number[] = [];
// const unitCircleY: number[] = [];
const circleResolution = 120;
// for (let i = 0; i < circleResolution; i++) {
//     unitCircleX[i] = Math.cos(2 * Math.PI * i / circleResolution);
//     unitCircleY[i] = Math.sin(2 * Math.PI * i / circleResolution);
// }

function create_explosion(id: number, x: number, y: number) {
    const explosion = explosionPool.pop();
    if (explosion) {
        explosion[0].visible = true;
        explosion[0].x = x;
        explosion[0].y = y;
        explosions.set(id, explosion);
    } else {
        const points: Point[] = [];
        for (let i = 0; i <= circleResolution; i++) {
            points.push(new Point(0, 0));
        }
        const rope = new SimpleRope(Texture.WHITE, points, 1 / 16);
        rope.tint = 0x000000;
        rope.x = x;
        rope.y = y;
        explosions.set(id, [rope, points]);
        app.stage.addChild(rope);
    }
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).create_explosion = create_explosion;

function render_explosion(id: number, radius: number, alpha: number) {
    const explosion = explosions.get(id);
    if (explosion) {
        explosion[0].alpha = alpha;
        const points = explosion[1];
        for (let i = 0; i <= circleResolution; i++) {
            points[i].x = radius * Math.cos(2 * Math.PI * i / circleResolution);
            points[i].y = radius * Math.sin(2 * Math.PI * i / circleResolution);
        }
    }
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).render_explosion = render_explosion;

function recycle_explosion(id: number) {
    const explosion = explosions.get(id);
    if (explosion) {
        explosion[0].visible = false;
        explosions.delete(id);
        explosionPool.push(explosion);
    }
}
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
(window as any).recycle_explosion = recycle_explosion;
