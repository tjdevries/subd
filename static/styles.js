import * as THREE from 'three';
import * as d3 from 'd3';
import mermaid from 'mermaid';

// Initialize Mermaid
mermaid.initialize({ startOnLoad: true });

console.log('Mermaid initialized');

// Function to create 3D animation using Three.js
function create3DScene() {
    console.log('Creating 3D scene');

    // Scene setup
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    // Sphere geometry
    const geometry = new THREE.SphereGeometry(5, 32, 32);
    const material = new THREE.MeshBasicMaterial({color: 0x00ff00, wireframe: true});
    const sphere = new THREE.Mesh(geometry, material);
    scene.add(sphere);

    camera.position.z = 15;
    camera.position.y = 5;

    console.log('3D sphere added to scene');

    // Animation loop
    function animate() {
        requestAnimationFrame(animate);
        sphere.rotation.x += 0.01;
        sphere.rotation.y += 0.01;
        renderer.render(scene, camera);
    }

    animate();
    console.log('Animation started');
}

// Function to create charts with Mermaid
function createCharts() {
    console.log('Creating Mermaid charts');
    const graphDefinition = `
        graph TD;
        A[Start] --> B{Is it working?};
        B -->|Yes| C[Great!];
        B -->|No| D[Try again];
    `;
    document.querySelector('.charts').innerHTML = mermaid.render('graphDiv', graphDefinition);
    console.log('Mermaid chart created');
}

// Function to create interactive visualization with D3.js
function createInteractiveVisualization() {
    console.log('Creating interactive visualization with D3');
    const svg = d3.select('body').append('svg')
        .attr('width', 960)
        .attr('height', 500);

    svg.append('circle')
        .attr('cx', 480)
        .attr('cy', 250)
        .attr('r', 100)
        .style('fill', 'coral')
        .on('mouseover', function() {
            d3.select(this).style('fill', 'purple');
            console.log('Circle color changed to purple');
        })
        .on('mouseout', function() {
            d3.select(this).style('fill', 'coral');
            console.log('Circle color reverted back to coral');
        });

    console.log('Interactive visualization created with D3.js');
}

console.log('Starting to create animated and interactive animations');
create3DScene();
createCharts();
createInteractiveVisualization();
console.log('All animations and interactions are set up');
// Save as styles.js file