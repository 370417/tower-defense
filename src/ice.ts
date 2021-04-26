import { Texture } from 'pixi.js';

const size = 64;

const canvas = document.createElement('canvas');
canvas.width = size;
canvas.height = size;
const ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
document.body.insertAdjacentElement('beforeend', canvas);

export const shieldTexture = Texture.from(canvas);

for (let y = 0; y < 64; y++) {
    for (let x = 0; x < 64; x++) {
        const dx = x - 64 / 2;
        const dy = y - 64 / 2;
        const theta = Math.atan2(dy, dx);
        const paint = 255 * (1 + theta / Math.PI) / 2;
        const distance = Math.sqrt(dx * dx + dy * dy) / 64;
        const distancePaint = 255 * Math.min(1, distance);
        ctx.fillStyle = `rgb(${paint},${distancePaint},${paint})`;
        ctx.fillRect(x, y, 1, 1);
    }
}

// const offscreenCanvas = document.createElement('canvas');
// offscreenCanvas.width = size;
// offscreenCanvas.height = size;
// const offscreenCtx = offscreenCanvas.getContext('2d') as CanvasRenderingContext2D;

// ctx.globalCompositeOperation = 'lighten';

// generateIceTexture(offscreenCtx, size, size, 'rgba(', ',0,0,1)');
// ctx.drawImage(offscreenCanvas, 0, 0);
// offscreenCtx.clearRect(0, 0, size, size);
// generateIceTexture(offscreenCtx, size, size, 'rgba(0,', ',0,1)');
// ctx.drawImage(offscreenCanvas, 0, 0);
// offscreenCtx.clearRect(0, 0, size, size);
// generateIceTexture(offscreenCtx, size, size, 'rgba(0,0,', ',1)');
// ctx.drawImage(offscreenCanvas, 0, 0);
// offscreenCtx.clearRect(0, 0, size, size);

function generateIceTexture(ctx: CanvasRenderingContext2D, width: number, height: number, colorPre: string, colorPost: string): void {
    const points = [{
        point: [width / 2, height / 2],
        isLeaf: true,
    }];

    ctx.globalCompositeOperation = 'destination-over';

    const iterations = 3000;

    for (let i = 0; i < 300; i++) {

        // Generate a random point
        const randPoint = randPointInCircle();
        let x = randPoint[0];
        let y = randPoint[1];
        x = width * (0.5 + 0.5 * x);
        y = height * (0.5 + 0.5 * y);

        // Find the closest point in the tree
        let closest = points[0];
        let closestDistSquared = distanceSquared([x, y], closest.point);
        for (let i = 0; i < points.length; i++) {
            const point = points[i];
            const dist2 = distanceSquared([x, y], point.point);
            if (dist2 < closestDistSquared) {
                closest = point;
                closestDistSquared = dist2;
            }
        }

        // Bring the random point close to that point
        let progress = i / iterations;
        progress = Math.pow(progress, 1 / Math.E);

        const threshold = 1 + 2 * (1 - progress);

        closest.isLeaf = false;
        const [closestX, closestY] = closest.point;
        const dist = Math.sqrt(closestDistSquared);
        const clampedDist = Math.min(threshold, dist);
        const dx = (x - closestX) * clampedDist / dist;
        const dy = (y - closestY) * clampedDist / dist;
        // const theta = Math.atan2(dy, dx);
        // const clampedTheta = Math.round(theta * 3 / Math.PI) * Math.PI / 3;
        const newx = closestX + dx;
        const newy = closestY + dy;

        // Add the random point to the tree now that its distance away has been clamped
        points.push({ point: [newx, newy], isLeaf: true });

        // // Add in between points
        // for (let len = 1; len <= clampedDist - 1; len++) {
        //     const dx = (x - closestX) * len / dist;
        //     const dy = (y - closestY) * len / dist;

        //     const newx = closestX + dx;
        //     const newy = closestY + dy;
        //     points.push([newx, newy]);
        // }

        ctx.strokeStyle = `${colorPre}${progress * 255 * 2}${colorPost}`;
        ctx.lineWidth = 3 - 2 * progress;
        ctx.lineCap = 'round';
        ctx.beginPath();
        ctx.moveTo(closestX, closestY);
        ctx.lineTo(newx, newy);
        ctx.stroke();
    }

    const leaves = points.filter(point => point.isLeaf);

    for (let i = 0; i < iterations * 2 / 3; i++) {

        // Generate a random point
        const randPoint = randPointInCircle();
        let x = randPoint[0];
        let y = randPoint[1];
        x = width * (0.5 + 0.5 * x);
        y = height * (0.5 + 0.5 * y);

        // Find the closest point in the tree
        let closest = leaves[0];
        let closestDistSquared = distanceSquared([x, y], closest.point);
        for (let i = 0; i < leaves.length; i++) {
            const point = leaves[i];
            const dist2 = distanceSquared([x, y], point.point);
            if (dist2 < closestDistSquared) {
                closest = point;
                closestDistSquared = dist2;
            }
        }

        // Bring the random point close to that point
        let progress = i / iterations;
        progress = Math.pow(progress, 1 / Math.E);

        const threshold = 1 + 2 * (1 - progress);

        closest.isLeaf = false;
        const [closestX, closestY] = closest.point;
        const dist = Math.sqrt(closestDistSquared);
        const clampedDist = Math.min(threshold, dist);
        const dx = (x - closestX) * clampedDist / dist;
        const dy = (y - closestY) * clampedDist / dist;
        const newx = closestX + dx;
        const newy = closestY + dy;

        // Add the random point to the tree now that its distance away has been clamped
        leaves.push({ point: [newx, newy], isLeaf: true });

        ctx.strokeStyle = `${colorPre}${progress * 255}${colorPost}`;
        ctx.lineWidth = 3 - 2 * progress;
        ctx.lineCap = 'round';
        ctx.beginPath();
        ctx.moveTo(closestX, closestY);
        ctx.lineTo(newx, newy);
        ctx.stroke();
    }
}

function distanceSquared([ax, ay]: number[], [bx, by]: number[]): number {
    return (ax - bx) * (ax - bx) + (ay - by) * (ay - by);
}

// Randomly pick a point in the unit circle with a uniform distribution.
function randPointInCircle(): [number, number] {
    const theta = 2 * Math.PI * Math.random();
    let r = Math.random() + Math.random();
    if (r > 1) {
        r = 2 - r;
    }
    return [r * Math.cos(theta), r * Math.sin(theta)];
}
