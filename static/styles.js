import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize mermaid for rendering markdown-like diagrams as charts
document.addEventListener('DOMContentLoaded', () => {
    mermaid.initialize({ startOnLoad: true });
    console.log('Mermaid initialized for chart rendering.');
});

// Set up a basic Three.js scene for fun floating 3D objects
function initializeThreeJSScene() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    camera.position.z = 5;

    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);
    console.log('Three.js renderer has been added to the page.');

    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);
    console.log('3D cube added to the scene.');

    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
        console.log('3D cube rotation updated.');
    }
    animate();
}

// Create a simple D3 chart for a dynamic visual representation of stats
function createD3Chart() {
    const data = [ {{ stats.ai_songs_count }}, {{ stats.ai_votes_count }} ];
    const svg = D3.select('body').append('svg').attr('width', 300).attr('height', 300);
    console.log('SVG element created for D3 chart.');
    
    svg.selectAll('rect')
       .data(data)
       .enter()
       .append('rect')
       .attr('x', (d, i) => i * 60)
       .attr('y', d => 300 - d)
       .attr('width', 50)
       .attr('height', d => d)
       .attr('fill', 'orange');
    console.log('D3 bar chart rendered with AI stats data.');
}

// Chart examples using Mermaid syntax
document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('.mermaid').forEach((element) => {
        element.innerHTML = `
            graph TD;
            A[Start] --> B{Is it?};
            B -->|Yes| C[Load 3D Scene];
            B -->|No| D[Load D3 Charts];
            C --> E[Enjoy Animation];
            D --> E;
        `;
    });
    console.log('Mermaid charts rendered.');
});

initializeThreeJSScene();
createD3Chart();
console.log('Animation and charts initialized successfully.');