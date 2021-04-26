import { bufferInput } from './input';

export const isPaused = false;
export const isSpedUp = false;

export function gameSpeed(): number {
    if (isPaused) {
        return 0;
    } else if (isSpedUp) {
        return 3;
    } else {
        return 1;
    }
}

const backButton = document.getElementsByClassName('back')[0] as HTMLButtonElement;
const playPauseButton = document.getElementsByClassName('pause')[0] as HTMLButtonElement;
const speedButton = document.getElementsByClassName('speed')[0] as HTMLButtonElement;
const nextButton = document.getElementsByClassName('next')[0] as HTMLButtonElement;

playPauseButton.addEventListener('click', playPause);
speedButton.addEventListener('click', toggleSpeed);

function playPause() {
    bufferInput({ type: 'play pause' });
    // if (isPaused) {
    //     isPaused = false;
    //     playPauseButton.classList.replace('play', 'pause');
    // } else {
    //     isPaused = true;
    //     playPauseButton.classList.replace('pause', 'play');
    // }
}

function toggleSpeed() {
    bufferInput({ type: 'fast forward' });
    // if (isSpedUp) {
    //     isSpedUp = false;
    //     speedButton.classList.remove('fast');
    // } else {
    //     isSpedUp = true;
    //     speedButton.classList.add('fast');
    // }
}

backButton.addEventListener('click', () => {
    bufferInput({ type: 'skip back' });
});

nextButton.addEventListener('click', () => {
    bufferInput({ type: 'send next wave' });
});
