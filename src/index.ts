/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import { Container, Filter, Graphics, Loader, ParticleContainer, Point, Renderer, SimpleRope, Sprite, Texture, Ticker } from 'pixi.js';
import { MAP_WIDTH, TILE_SIZE, MAP_HEIGHT, MS_PER_UPDATE, MAX_UPDATES_PER_TICK } from './constants';
import { initGridInput } from './input';
import { drawGrid, initPathRendering } from './render/grid';
import { initRangeRendering } from './render/range';
import './settings';

const rendererContainer = document.getElementById('grid') as HTMLDivElement;

const loadBackend = import('../dist/tower_defense');
const loadMemory = import('../dist/tower_defense_bg.wasm');

const loader = Loader.shared;
const ticker = Ticker.shared;
const stage = new Container();
let renderer = new Renderer({
    width: MAP_WIDTH * TILE_SIZE + 1,
    height: MAP_HEIGHT * TILE_SIZE + 1,
    backgroundColor: 0xFFFFFF,
    antialias: true,
    resolution: window.devicePixelRatio,
    autoDensity: true,
});
renderer.render(stage);
rendererContainer.insertAdjacentElement('afterbegin', renderer.view);

export function refreshRenderer(antialias: boolean, resolution: number): void {
    rendererContainer.removeChild(renderer.view);
    renderer = new Renderer({
        width: MAP_WIDTH * TILE_SIZE + 1,
        height: MAP_HEIGHT * TILE_SIZE + 1,
        backgroundColor: 0xFFFFFF,
        antialias,
        resolution,
        autoDensity: true,
    });
    rendererContainer.insertAdjacentElement('afterbegin', renderer.view);
}

loader
    .add('ice_shader', 'ice_shader.frag')
    .add('spritesheet', 'texture-atlas.json')
    .load((loader, resources) => {

        const spritesheet = resources.spritesheet?.spritesheet;

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const iceShader = resources.ice_shader?.data;

        if (
            !iceShader ||
            !spritesheet) {
            return;
        }

        const swallowTexture = spritesheet.textures['swallow.png'] as Texture;
        const circleTexture = spritesheet.textures['circle.png'] as Texture;
        const falconTexture = spritesheet.textures['falcon.png'] as Texture;
        const indicatorTexture = spritesheet.textures['exclamation.png'] as Texture;
        const missileTexture = spritesheet.textures['missile.png'] as Texture;
        const whiteTexture = spritesheet.textures['white.png'] as Texture;

        // Organize visuals by layer

        const background = new ParticleContainer();
        stage.addChild(background);

        const towerLayer = new Container();
        stage.addChild(towerLayer);

        const smokeLayer = new Container();
        stage.addChild(smokeLayer);

        const projectileLayer = new Container();
        stage.addChild(projectileLayer);

        const enemyLayer = new Container();
        stage.addChild(enemyLayer);

        const spriteLayer = new Container();
        stage.addChild(spriteLayer);

        const rangeLayer = new Container();
        stage.addChild(rangeLayer);

        initPathRendering(background);
        initRangeRendering(rangeLayer);

        const graphics = new Map<number, Container>();

        function create_mob(id: number) {
            const mob = new Graphics();
            mob.beginFill(0x666666);
            mob.drawCircle(0, 0, 0.3 * TILE_SIZE);
            enemyLayer.addChild(mob);
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

        const towers = new Map<number, [Sprite, Sprite]>();

        function create_tower(id: number, row: number, col: number) {
            const border = Sprite.from(Texture.WHITE);
            border.tint = 0x000000;
            border.width = TILE_SIZE + 1;
            border.height = TILE_SIZE + 1;
            border.x = col * TILE_SIZE;
            border.y = row * TILE_SIZE;
            towerLayer.addChild(border);

            const background = Sprite.from(Texture.WHITE);
            background.tint = 0xF8F8F8;
            background.width = TILE_SIZE - 1;
            background.height = TILE_SIZE - 1;
            background.x = col * TILE_SIZE + 1;
            background.y = row * TILE_SIZE + 1;
            towerLayer.addChild(background);

            towers.set(id, [border, background]);
        }
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
        (window as any).create_tower = create_tower;

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
                smokeLayer.addChild(rope);
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

        const circleResolution = 50;

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
                smokeLayer.addChild(rope);
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

        drawGrid(background);

        Promise.all([loadBackend, loadMemory]).then(([worldModule, memModule]) => {

            // Some graphics functions read wasm memory, so we bind those parameters

            // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
            (window as any).render_smoke_trail = render_smoke_trail.bind(undefined, memModule.memory);

            const sprites: Sprite[] = [];

            const world = worldModule.World.new();

            function render(frameFudge: number) {
                world.dump_sprite_data(frameFudge);

                const spriteCount = world.sprite_count();
                const spriteIds = new Uint8Array(memModule.memory.buffer, world.sprite_id(), spriteCount);
                const spriteXs = new Float32Array(memModule.memory.buffer, world.sprite_x(), spriteCount);
                const spriteYs = new Float32Array(memModule.memory.buffer, world.sprite_y(), spriteCount);
                const spriteRotations = new Float32Array(memModule.memory.buffer, world.sprite_rotation(), spriteCount);
                const spriteAlphas = new Float32Array(memModule.memory.buffer, world.sprite_alpha(), spriteCount);
                const spriteTints = new Uint32Array(memModule.memory.buffer, world.sprite_tint(), spriteCount);

                for (let i = sprites.length; i < spriteCount; i++) {
                    const sprite = new Sprite(swallowTexture);
                    sprites.push(sprite);
                    spriteLayer.addChild(sprite);
                }
                for (let i = 0; i < spriteCount; i++) {
                    const sprite = sprites[i];
                    sprite.visible = true;
                    switch (spriteIds[i]) {
                        case 0:
                            sprite.texture = swallowTexture;
                            sprite.width = 0.8 * TILE_SIZE;
                            sprite.height = 0.8 * TILE_SIZE;
                            sprite.anchor.set(0.5, 0.5);
                            break;
                        case 1:
                            sprite.texture = missileTexture;
                            sprite.width = 16;
                            sprite.height = 8;
                            sprite.anchor.set(0.5, 0.5);
                            break;
                        case 2:
                            sprite.texture = falconTexture;
                            sprite.width = 0.75 * TILE_SIZE;
                            sprite.height = 0.75 * TILE_SIZE;
                            sprite.anchor.set(0.5, 0.5);
                            break;
                        case 3:
                            sprite.texture = circleTexture;
                            sprite.width = 0.6 * TILE_SIZE;
                            sprite.height = 0.6 * TILE_SIZE;
                            sprite.anchor.set(0.5, 0.5);
                            break;
                        case 4:
                            sprite.texture = indicatorTexture;
                            sprite.width = 0.5 * TILE_SIZE;
                            sprite.height = 0.5 * TILE_SIZE;
                            sprite.anchor.set(0.5, 1.0);
                            break;
                        case 5:
                            sprite.texture = whiteTexture;
                            sprite.width = 10;
                            sprite.height = 10;
                            sprite.anchor.set(0.5, 0.5);
                            break;
                    }
                    sprite.x = spriteXs[i];
                    sprite.y = spriteYs[i];
                    sprite.rotation = spriteRotations[i];
                    sprite.alpha = spriteAlphas[i];
                    sprite.tint = spriteTints[i];
                }
                for (let i = spriteCount; i < sprites.length; i++) {
                    sprites[i].visible = false;
                }
            }

            // Input
            const mouseHoverPos = {
                row: -1,
                col: -1,
            };
            initGridInput(rendererContainer, mouseHoverPos, []);

            const filter = new Filter(undefined, iceShader, {
                customUniform: 0.5,
            });

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
                    // world.render(0); no point in rendering here?
                    // Don't even process input. Is this a good idea?
                    return;
                }

                if (mouseHoverPos.row >= 0 && mouseHoverPos.col >= 0) {
                    world.hover_map(0, mouseHoverPos.row, mouseHoverPos.col);
                }

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
                render(lag / MS_PER_UPDATE);
                renderer.render(stage);
            }

            ticker.add(gameTick);
        }).catch(console.error);

    });
