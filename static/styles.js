import * as THREE from 'three';
import * as D3 from 'd3';
import mermaid from 'mermaid';

// Page initialization
console.log("Initializing page...");
document.addEventListener('DOMContentLoaded', () => {
    setupThreeJSScene();
    setupMermaidChart();
    enhanceNavLinks();
    applyRainbowHeaderAnimation();
});

// THREE.js Scene setup for playful 3D header animation
function setupThreeJSScene() {
    console.log("Setting up Three.js scene for the rainbow header...");
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / 200, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer({ alpha: true });

    renderer.setSize(window.innerWidth, 200);
    const header = document.querySelector('.header-container');
    header.appendChild(renderer.domElement);

    const geometry = new THREE.TorusKnotGeometry(10, 3, 100, 16);
    const material = new THREE.MeshBasicMaterial({ color: 0xffff00 });
    const torusKnot = new THREE.Mesh(geometry, material);
    scene.add(torusKnot);

    camera.position.z = 30;

    const animate = function () {
        requestAnimationFrame(animate);

        torusKnot.rotation.x += 0.01;
        torusKnot.rotation.y += 0.01;

        renderer.render(scene, camera);
    };
    animate();
    console.log("3D animation initialized!");
}

// Animating the navigation links with color shifts
function enhanceNavLinks() {
    console.log("Adding animation to navigation links...");
    const navLinks = document.querySelectorAll('.nav-link');

    navLinks.forEach((link, index) => {
        link.style.transition = "color 0.5s ease, transform 0.5s ease";
        link.style.color = `hsl(${(360 / navLinks.length) * index}, 80%, 60%)`;

        link.addEventListener('mouseover', () => {
            link.style.color = '#fff';
            link.style.transform = 'scale(1.1)';
        });

        link.addEventListener('mouseout', () => {
            link.style.color = `hsl(${(360 / navLinks.length) * index}, 80%, 60%)`;
            link.style.transform = 'scale(1)';
        });
    });
    console.log("Navigation animation set!");
}

// Mermaid.js chart example
function setupMermaidChart() {
    console.log("Initializing Mermaid.js chart...");
    const chartDiv = document.createElement('div');
    chartDiv.innerHTML = `
    <div class="mermaid">
        graph TD;
        AI-Generated-->Rainbow;
        Rainbow-->Music;
        Music-->Charts;
        Charts-->Fun;
    </div>`;
    document.body.appendChild(chartDiv);

    mermaid.initialize({ theme: 'default' });
    mermaid.contentLoaded();
    console.log("Mermaid chart rendered successfully!");
}

// Rainbow color animation for headers
function applyRainbowHeaderAnimation() {
    console.log("Applying rainbow effect to header...");
    const headers = document.querySelectorAll('h1, h2');

    headers.forEach(header => {
        header.style.background = 'linear-gradient(45deg, red, orange, yellow, green, blue, indigo, violet)';
        header.style.backgroundSize = '400% 100%';
        header.style.webkitBackgroundClip = 'text';
        header.style.webkitTextFillColor = 'transparent';

        let degree = 0;
        setInterval(() => {
            degree += 5;
            header.style.backgroundPosition = `${degree % 400}%`;
        }, 100);
    });
    console.log("Rainbow header animation in place!");
}