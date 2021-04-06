const side = document.getElementById('side') as HTMLDivElement;
const towers = document.getElementsByClassName('tower');

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
        side.dataset.tower = towerNames[i];
    });

    towers[i].addEventListener('mouseleave', () => {
        if (side.dataset.clickedTower) {
            side.dataset.tower = side.dataset.clickedTower;
        }
    });

    towers[i].addEventListener('click', () => {
        if (side.dataset.clickedTower === towerNames[i]) {
            side.dataset.tower = '';
            side.dataset.clickedTower = '';
        } else {
            side.dataset.tower = towerNames[i];
            side.dataset.clickedTower = towerNames[i];
        }
    });
}

side.addEventListener('mouseleave', () => {
    side.dataset.tower = side.dataset.clickedTower || '';
});
