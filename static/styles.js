import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

console.log('Initializing Symphony of Chaos Animation');

/** THREE.js Animation **/
function createThreeJSScene() {
    console.log('Setting up THREE.js scene');
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    camera.position.z = 5;
    console.log('Cube added to scene');

    function animate() {
        requestAnimationFrame(animate);

        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;

        renderer.render(scene, camera);
    }

    animate();
}

/** D3.js Animation **/
function createD3Chart() {
    console.log('Creating D3.js chart');
    const data = [12, 5, 6, 6, 9, 10];
    const svg = D3.select('body')
        .append('svg')
        .attr('width', 300)
        .attr('height', 200);

    svg.selectAll('rect')
        .data(data)
        .enter()
        .append('rect')
        .attr('x', (d, i) => i * 45)
        .attr('y', (d, i) => 200 - (d * 10))
        .attr('width', 40)
        .attr('height', (d, i) => d * 10)
        .attr('fill', 'orange');

    console.log('D3 chart rendered');
}

/** Mermaid.js Diagram **/
function renderMermaidChart() {
    console.log('Rendering mermaid diagram');
    const chartDefinition = `graph TD;
    A[Hard start] -->B{Is it?};
    B -- Yes --> C[Good];
    B -- No --> D[Bad];
    C --> E[/Fine/];
    D --> E;`;

    mermaid.render('diagram', chartDefinition, (svgCode) => {
        const div = document.createElement('div');
        div.innerHTML = svgCode;
        document.body.appendChild(div);
    });

    console.log('Mermaid chart added to document');
}

/** Init methods **/
window.onload = function() {
    console.log('Window loaded, starting animations');
    createThreeJSScene();
    createD3Chart();
    renderMermaidChart();
    console.log('All animations initialized');
}