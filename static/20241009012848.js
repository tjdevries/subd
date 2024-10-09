// Importing necessary libraries
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';

// Create the scene
globalThis.scene = new THREE.Scene();

// Create the camera
globalThis.camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
globalThis.camera.position.z = 5;

// Create the renderer
globalThis.renderer = new THREE.WebGLRenderer({ antialias: true });
globalThis.renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(globalThis.renderer.domElement);

// Create a water surface plane
globalThis.waterGeometry = new THREE.PlaneGeometry(20, 20, 32, 32);
globalThis.waterMaterial = new THREE.MeshPhongMaterial({
    color: 0x001e0f,
    side: THREE.DoubleSide,
    wireframe: false
});
globalThis.water = new THREE.Mesh(globalThis.waterGeometry, globalThis.waterMaterial);
globalThis.water.rotation.x = Math.PI / 2;
globalThis.scene.add(globalThis.water);

// Create Blastoise character using a placeholder object
globalThis.blastoiseGeometry = new THREE.SphereGeometry(1, 32, 32);
globalThis.blastoiseMaterial = new THREE.MeshNormalMaterial();
globalThis.blastoise = new THREE.Mesh(globalThis.blastoiseGeometry, globalThis.blastoiseMaterial);
globalThis.blastoise.position.y = 1;
globalThis.scene.add(globalThis.blastoise);

// Create lights
globalThis.ambientLight = new THREE.AmbientLight(0x404040, 3);
globalThis.scene.add(globalThis.ambientLight);

globalThis.directionalLight = new THREE.DirectionalLight(0xffffff, 2);
globalThis.directionalLight.position.set(5, 10, 7.5);
globalThis.scene.add(globalThis.directionalLight);

// Add controls
globalThis.controls = new OrbitControls(globalThis.camera, globalThis.renderer.domElement);
globalThis.controls.enableDamping = true;
globalThis.controls.dampingFactor = 0.1;

// Animation cycle
globalThis.animate = function () {
    requestAnimationFrame(globalThis.animate);
    globalThis.blastoise.rotation.x += 0.01;
    globalThis.blastoise.rotation.y += 0.01;
    globalThis.controls.update();
    globalThis.renderer.render(globalThis.scene, globalThis.camera);
};

// Responsive design
globalThis.onWindowResize = function () {
    globalThis.camera.aspect = window.innerWidth / window.innerHeight;
    globalThis.camera.updateProjectionMatrix();
    globalThis.renderer.setSize(window.innerWidth, window.innerHeight);
};

window.addEventListener('resize', globalThis.onWindowResize, false);

globalThis.animate();