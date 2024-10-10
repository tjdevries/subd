// Importing necessary libraries from import map
import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize mermaid.js for chart rendering
mermaid.initialize({ startOnLoad: true });

// Log to console each step of the JavaScript magic
console.log("JavaScript and libraries imported. Let's create some magic!");

// Function to animate header with a rotating rainbow effect
function animateRainbowHeader() {
    const header = document.querySelector('.header-container h1');
    let hue = 0;
    setInterval(() => {
        header.style.color = `hsl(${hue}, 100%, 50%)`;
        hue = (hue + 1) % 360;
    }, 50); // Rotate hue to create a rainbow transition
    console.log("Animating the header with a rotating rainbow!");
}

// Function to create a 3D bouncing cube with three.js
function create3DBouncingCube() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    camera.position.z = 5;

    const animate = function () {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        cube.position.y = Math.sin(Date.now() * 0.001) * 0.5;
        renderer.render(scene, camera);
    };

    animate();
    console.log("3D Bouncing cube created and animated!");
}

// Function to generate charts with mermaid.js
function generateCharts() {
    const topSongsChart = `
    graph LR
    A[Top Songs] --> B{{"Song 1"}}
    A --> C{{"Song 2"}}
    A --> D{{"Song 3"}}
    B --> B1["Score: 9.4"]
    C --> C1["Score: 8.8"]
    D --> D1["Score: 8.5"]
    `;

    const div = document.createElement('div');
    div.className = 'mermaid';
    div.innerHTML = topSongsChart;
    document.querySelector('.charts-section').appendChild(div);
    mermaid.contentLoaded();
    console.log("Mermaid.js chart of top songs generated!");
}

// Add event listeners to nav links for smooth scrolling to sections
document.querySelectorAll('.nav-link').forEach(link => {
    link.addEventListener('click', function(e) {
        const targetId = this.getAttribute('href').substring(1);
        const targetSection = document.getElementById(targetId);
        if (targetSection) {
            e.preventDefault();
            targetSection.scrollIntoView({ behavior: 'smooth' });
            console.log(`Smooth scroll to section: ${targetId}`);
        }
    });
});

// Call the functions to initialize animations and charts
animateRainbowHeader();
create3DBouncingCube();
generateCharts();
console.log("All animations and charts are set! Enjoy the interactivity and visuals!");
