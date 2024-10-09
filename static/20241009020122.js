// Import necessary libraries
import * as THREE from 'three';
import Phaser from 'phaser';
import { TweenMax } from 'gsap';

// Initialize Three.js scene
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Create a moon using Three.js
const geometry = new THREE.SphereGeometry(5, 32, 32);
const material = new THREE.MeshBasicMaterial({ color: 0xffffff });
const moon = new THREE.Mesh(geometry, material);
scene.add(moon);
camera.position.z = 10;

// Add some interactive lighting
const light = new THREE.PointLight(0x404040);
light.position.set(10, 10, 10).normalize();
scene.add(light);

// Rendering the scene
function animate() {
	requestAnimationFrame(animate);
	renderer.render(scene, camera);
}
animate();

// Add animations using GSAP
TweenMax.to(moon.rotation, 10, { y: 360, repeat: -1, ease: 'linear' });

// Create dynamic hover animation for playlist songs
const links = document.querySelectorAll('.unplayed_song a');

links.forEach(link => {
	link.addEventListener('mouseover', () => {
		link.style.transition = '0.3s';
		links.forEach(other => {
			if (other !== link) {
				other.style.opacity = '0.3';
			}
		});
		link.style.transform = 'scale(1.1)';
	});
	link.addEventListener('mouseout', () => {
		links.forEach(other => {
			other.style.opacity = '1';
		});
		link.style.transform = 'scale(1)';
	});
});

// Animations for current song details
const currentSongInfo = document.querySelector('.current-song-info');
currentSongInfo.style.opacity = 0;
window.onload = () => {
	TweenMax.to(currentSongInfo, 2, {opacity: 1});
};

// Adding animation effects for nav-links
const navLinks = document.querySelectorAll('.nav-link');

navLinks.forEach(link => {
	link.addEventListener('mouseover', () => {
		TweenMax.to(link, 1, { color: '#ff69b4' });
	});
	link.addEventListener('mouseout', () => {
		TweenMax.to(link, 0.5, { color: '#000' });
	});
});

// Initialize Phaser game for additional animations
const config = {
	type: Phaser.AUTO,
	width: 800,
	height: 600,
	physics: {
		default: 'arcade',
		arcade: {
			gravity: { y: 300 },
			enableBody: true
		}
	},
	scene: {
		preload: preload,
		create: create
	}
};

const game = new Phaser.Game(config);

function preload() {
	this.load.setBaseURL('http://labs.phaser.io');
	this.load.image('sky', 'assets/skies/space3.png');
}

function create() {
	this.add.image(400, 300, 'sky');
}