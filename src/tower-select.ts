const side = document.getElementById('side') as HTMLDivElement;
const towers = document.getElementsByClassName('tower');

export let clickedTower: TowerSelectionState | undefined = undefined;
export let hoveredTower: TowerSelectionState | undefined = undefined;

export type TowerSelectionState = {
    towerName: string;
    towerIndex: number;
    towerEntity: number | undefined;
    towerStatus: 'prototype' | 'queued' | 'building' | 'operational' | 'upgrading';
};

export let selectedTowerIsDirty = false;

export function selectedTower(): number {
    return towerNames.indexOf(side.dataset.tower || '');
}

const towerNames = [
    'swallow',
    'tesla',
    'fire',
    'tree',
    'falcon',
    'gauss',
    'missile',
    'factory',
];

for (let i = 0; i < towers.length; i++) {
    towers[i].addEventListener('mouseenter', () => {
        selectedTowerIsDirty = true;
        side.dataset.tower = towerNames[i];
        hoveredTower = {
            towerName: towerNames[i],
            towerIndex: i,
            towerEntity: undefined,
            towerStatus: 'prototype',
        };
    });

    towers[i].addEventListener('mouseleave', () => {
        hoveredTower = undefined;
        if (clickedTower) {
            selectedTowerIsDirty = true;
            side.dataset.tower = clickedTower.towerName;
        }
    });

    towers[i].addEventListener('click', () => {
        selectedTowerIsDirty = true;
        if (clickedTower && clickedTower.towerName === towerNames[i] && clickedTower.towerStatus === 'prototype') {
            clickedTower = undefined;
            side.dataset.tower = '';
        } else {
            clickedTower = {
                towerName: towerNames[i],
                towerIndex: i,
                towerEntity: undefined,
                towerStatus: 'prototype',
            };
            side.dataset.tower = clickedTower.towerName;
        }
    });
}

side.addEventListener('mouseleave', () => {
    hoveredTower = undefined;
    side.dataset.tower = clickedTower?.towerName || '';
});

const towerTitle = document.getElementById('tower-title') as HTMLDivElement;
const towerTitleBg = document.querySelector('#tower-stats .tower-name') as HTMLDivElement;
const towerCost = document.getElementById('tower-cost') as HTMLSpanElement;
const damageVal = document.querySelector('#damage .val') as HTMLSpanElement;
const damageBar = document.querySelector('#damage .barline') as HTMLDivElement;
const rateOfFireLabel = document.getElementById('rate-of-fire-title') as HTMLSpanElement;
const rateOfFireVal = document.querySelector('#rate-of-fire .val') as HTMLSpanElement;
const rateOfFireBar = document.querySelector('#rate-of-fire .barline') as HTMLDivElement;
const rangeVal = document.querySelector('#range .val') as HTMLSpanElement;
const rangeBar = document.querySelector('#range .barline') as HTMLDivElement;
const description = document.getElementById('description') as HTMLParagraphElement;
const flavor = document.getElementById('flavor') as HTMLElement;

export function renderTowerSelect(
    title: string,
    damage: number,
    cost: number,
    rateOfFire: number,
    range: number,
    descStr: string,
    flavorStr: string,
): void {
    selectedTowerIsDirty = false;
    towerTitle.textContent = title;
    if (title === 'Swallow' || title === 'Falcon')
        towerTitleBg.style.backgroundColor = '#d4e8ee';
    if (title === 'Fire' || title === 'Missile')
        towerTitleBg.style.backgroundColor = '#f5bec5';
    if (title === 'Tree' || title === 'Factory')
        towerTitleBg.style.backgroundColor = '#c0e6bf';
    if (title === 'Tesla' || title === 'Gauss')
        towerTitleBg.style.backgroundColor = '#eedcba';
    towerCost.textContent = `cost ${formatFloat(cost)}s`;
    damageVal.textContent = formatFloat(damage);
    if (title === 'Fire')
        damageVal.textContent += 'Hz';
    damageBar.style.background = formatGradient(damage, 0, 100);
    rateOfFireLabel.textContent = title === 'Swallow' ? 'Airspeed' : 'Rate of fire';
    rateOfFireVal.textContent = formatFloat(rateOfFire);
    rateOfFireBar.style.background = formatGradient(rateOfFire, 0, 100);
    rangeVal.textContent = formatFloat(range);
    rangeBar.style.background = formatGradient(range, 0, 15);
    description.textContent = descStr;
    flavor.textContent = flavorStr;
}

export function cancelTowerSelect(): void {
    clickedTower = undefined;
    hoveredTower = undefined;
    side.dataset.tower = '';
}

function formatFloat(n: number): string {
    if (!Number.isFinite(n)) {
        return '∞';
    }
    if (n % 1 < 0.1) {
        return n.toFixed(0);
    } else {
        return n.toFixed(1);
    }
}

function formatGradient(stop0: number, min: number, max: number): string {
    stop0 = Math.min(Math.max(stop0, min), max);
    const stop0percentage = 100 * (stop0 - min) / (max - min);
    return `linear-gradient(to right, #000 ${stop0percentage}%, #e8e8e8 ${stop0percentage}%)`;
}
