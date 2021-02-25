import { Application, Container, Graphics, Sprite, Texture } from 'pixi.js';
import { MAP_WIDTH, TILE_SIZE, MAP_HEIGHT, TRUE_MAP_WIDTH, MS_PER_UPDATE, MAX_UPDATES_PER_TICK } from './constants';
import { Components, Missile, MissileSpawner, Mob, SmokeParticle, SmokeTrail, State, Swallow, Tower, Velocity } from './state';
import { operateMissileTower, updateMissile } from './systems/missile';
import { renderSmoke, spawnSmokeParticles, updateSmokeParticles } from './systems/missile-smoke';
import { createMob, moveMobs } from './systems/move-mobs';
import { renderMobPositions } from './systems/render-mob-positions';
import { baseTowerSprite } from './systems/render-towers';
import { createSwallowTower } from './systems/swallow';
import { executeWalk, loopWalkers, planWalk } from './systems/walker';

const app = new Application({
    width: MAP_WIDTH * TILE_SIZE + 1,
    height: MAP_HEIGHT * TILE_SIZE + 1,
    backgroundColor: 0xFFFFFF,
    antialias: true,
});

document.body.appendChild(app.view);

app.loader.load((loader, resources) => {
    let lastUpdateTime = window.performance.now();
    let lag = 0;

    // Draw the grid

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

    // Define the path

    // > means go left
    // < means go right
    // n means go up
    // v means go down
    // x means turn

    const path = [
        '##########################',
        '##########################',
        '##                      ##',
        '>>>>x x>>>x    x>>>x x>>>>',
        '>>>xv nx>xv    nx>xv nx>>>',
        '## vx>xn vv    nn vx>xn ##',
        '## x>>>x vv    nn x>>>x ##',
        '##       vv    nn       ##',
        '## x<<<x vv    nn x<<<x ##',
        '## vx<xn vv    nn vx<xn ##',
        '## vv nx<xv    nx<xv nn ##',
        '## vv x<<<x    x<<<x nn ##',
        '## vv                nn ##',
        '## vv                nn ##',
        '## vv  x>>>x  x>>>x  nn ##',
        '## vv  nx>xv  nx>xv  nn ##',
        '## vv  nn vx>>xn vv  nn ##',
        '## vx>>xn x>>>>x vx>>xn ##',
        '## x>>>>x        x>>>>x ##',
        '##                      ##',
        '##########################',
        '##########################',
    ];

    // Define components

    const components: Components = {
        mobs: new Map<number, Mob>(),
        sprites: new Map<number, Container>(),
        walkVelocity: new Map<number, Velocity>(),
        towers: new Map<number, Tower>(),
        missiles: new Map<number, Missile>(),
        missileSpawners: new Map<number, MissileSpawner>(),
        smokeParticles: new Map<number, SmokeParticle>(),
        smokeTrails: new Map<number, SmokeTrail>(),
        swallows: new Map<number, Swallow>(),
    };

    const state: State = {
        tick: 0,
        map: {
            path,
            entrances: [
                { row: 3, col: 0 },
                { row: 4, col: 0 },
            ],
            exits: [
                { row: 3, col: TRUE_MAP_WIDTH - 1 },
                { row: 4, col: TRUE_MAP_WIDTH - 1 },
            ],
        },
        components,
        nextEntity: 0,
    };

    // Draw the path

    for (let row = 0; row < MAP_HEIGHT; row++) {
        for (let col = 0; col < MAP_WIDTH; col++) {
            if (path[row + 2][col + 2] !== ' ') {
                const square = Sprite.from(Texture.WHITE);
                square.tint = 0xE8E8E8;
                square.width = TILE_SIZE + 1;
                square.height = TILE_SIZE + 1;
                square.x = col * TILE_SIZE;
                square.y = row * TILE_SIZE;
                app.stage.addChild(square);
            }
        }
    }

    // Draw the path border

    // Horizontal edges

    for (let row = 1; row < MAP_HEIGHT; row++) {
        for (let col = 0; col < MAP_WIDTH; col++) {
            if ((path[row + 2][col + 2] === ' ') !== (path[row + 1][col + 2] === ' ')) {
                const edge = Sprite.from(Texture.WHITE);
                edge.tint = 0x000000;
                edge.width = TILE_SIZE + 1;
                edge.height = 1;
                edge.x = col * TILE_SIZE;
                edge.y = row * TILE_SIZE;
                app.stage.addChild(edge);
            }
        }
    }

    // Vertical edges

    for (let row = 0; row < MAP_HEIGHT; row++) {
        for (let col = 1; col < MAP_WIDTH; col++) {
            if ((path[row + 2][col + 2] === ' ') !== (path[row + 2][col + 1] === ' ')) {
                const edge = Sprite.from(Texture.WHITE);
                edge.tint = 0x000000;
                edge.width = 1;
                edge.height = TILE_SIZE + 1;
                edge.x = col * TILE_SIZE;
                edge.y = row * TILE_SIZE;
                app.stage.addChild(edge);
            }
        }
    }

    // Add some demo enemies

    function createEnemy(x: number, y: number) {
        state.components.mobs.set(state.nextEntity, createMob(x, y));

        state.components.walkVelocity.set(state.nextEntity, {
            dx: 0,
            dy: 0,
        });

        const dummy = new Graphics();
        dummy.beginFill(0x008800);
        dummy.drawCircle(0, 0, 0.3 * TILE_SIZE);
        dummy.x = x;
        dummy.y = y;
        app.stage.addChild(dummy);

        state.components.sprites.set(state.nextEntity, dummy);

        state.nextEntity += 1;
    }

    createEnemy(-TILE_SIZE, 2.5 * TILE_SIZE);
    createEnemy(-TILE_SIZE, 1.5 * TILE_SIZE);

    // Add some demo towers

    function createTower(row: number, col: number) {
        const sprite = baseTowerSprite();
        sprite.x = col * TILE_SIZE;
        sprite.y = row * TILE_SIZE;
        app.stage.addChild(sprite);

        state.components.towers.set(state.nextEntity, { row, col });
        state.components.missileSpawners.set(state.nextEntity, {
            reloadCost: 90,
            reloadCountdown: 60,
        });

        state.nextEntity += 1;
    }

    createTower(3, 6);
    createTower(2, 10);

    createSwallowTower(state, app, 7, 6);
    createSwallowTower(state, app, 8, 3);

    /** Game loop with fixed time step, variable rendering */
    function gameTick() {
        const time = window.performance.now();
        const elapsed = time - lastUpdateTime;
        lastUpdateTime = time;
        lag += elapsed;

        if (lag > MS_PER_UPDATE * MAX_UPDATES_PER_TICK) {
            // Too much lag, just pretend it doesn't exist
            lag = 0;
            render(0);
            // Don't even process input. Is this a good idea?
            return;
        }

        // process input

        let updates = 0;
        while (lag >= MS_PER_UPDATE) {
            update();
            updates += 1;
            lag -= MS_PER_UPDATE;
            if (updates > MAX_UPDATES_PER_TICK) {
                render(1);
                return;
            }
        }

        render(lag / MS_PER_UPDATE);
    }

    function update() {
        updateSmokeParticles(state);
        updateMissile(state, app);
        spawnSmokeParticles(state, app);
        operateMissileTower(state, app);
        planWalk(state.components, state.map.path);
        executeWalk(state.components);
        moveMobs(state.components);
        loopWalkers(state.components, state);

        state.tick += 1;
    }

    function render(timeTillRender: number) {
        renderMobPositions(state.components, timeTillRender);
        renderSmoke(state);
    }

    app.ticker.add(gameTick);
});
