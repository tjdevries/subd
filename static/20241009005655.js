// JavaScript using three.js for creating a fun, interactive background
import * as THREE from 'three';

function createBackgroundAnimation() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    // Add colorful spinning cubes representing the chaos of modals
    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    cube.position.x = Math.random() * 10 - 5;
    cube.position.y = Math.random() * 10 - 5;
    cube.position.z = Math.random() * 10 - 5;

    camera.position.z = 5;

    const animate = function () {
        requestAnimationFrame(animate);

        // Dynamic cube movement and rotation
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        cube.position.y += 0.01 * Math.sin(Date.now() * 0.001);

        renderer.render(scene, camera);
    };

    animate();
}

window.onload = createBackgroundAnimation;