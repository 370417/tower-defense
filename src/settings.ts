import { refreshRenderer } from '.';

const antialiasInput = document.getElementById('antialias') as HTMLInputElement;
const resolutionSelect = document.getElementById('resolution') as HTMLSelectElement;

function updateVideoSettings() {
    let resolution = 1;
    if (resolutionSelect.value === 'Native') resolution = window.devicePixelRatio;
    if (resolutionSelect.value === 'High') resolution = 2;
    refreshRenderer(antialiasInput.checked, resolution);
}

antialiasInput.addEventListener('input', updateVideoSettings);
resolutionSelect.addEventListener('input', updateVideoSettings);

resolutionSelect.children[0].textContent += ` (${window.devicePixelRatio}x)`;

const imageRenderingSelect = document.getElementById('image-rendering') as HTMLSelectElement;

imageRenderingSelect.addEventListener('input', () => {
    const style = document.getElementById('canvas-style') as Element;
    style.textContent = `canvas { image-rendering: ${imageRenderingSelect.value} }`;
});
