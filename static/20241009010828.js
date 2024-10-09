function init3DEffects() {
  if (typeof THREE !== 'undefined') {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.SphereGeometry(1, 32, 32);
    const material = new THREE.MeshBasicMaterial({ color: 0xffff00 });
    const sphere = new THREE.Mesh(geometry, material);
    scene.add(sphere);

    camera.position.z = 5;

    function animate() {
      requestAnimationFrame(animate);
      sphere.rotation.x += 0.01;
      sphere.rotation.y += 0.01;
      renderer.render(scene, camera);
    }

    animate();
  } else {
    console.error('Three.js is not loaded');
  }
}

document.addEventListener('DOMContentLoaded', () => {
  init3DEffects();
});

// Custom animations for interactive elements
function addInteractiveAnimations() {
  const songLinks = document.querySelectorAll('.unplayed_song a, .song_link a');
  songLinks.forEach(link => {
    link.addEventListener('mouseenter', () => {
      link.style.transition = 'transform 0.5s';
      link.style.transform = 'scale(1.1)';
    });
    link.addEventListener('mouseleave', () => {
      link.style.transform = 'scale(1)';
    });
  });
}
document.addEventListener('DOMContentLoaded', addInteractiveAnimations);

// Phaser.js animation for a fun mini background game
function startPhaserGame() {
  if (typeof Phaser !== 'undefined') {
    const config = {
      type: Phaser.AUTO,
      width: 800,
      height: 600,
      physics: {
        default: 'arcade',
        arcade: {
          gravity: { y: 200 }
        }
      },
      scene: {
        preload: preload,
        create: create
      }
    };
    const game = new Phaser.Game(config);

    function preload() {
      this.load.setBaseURL('https://labs.phaser.io');
      this.load.image('sky', 'assets/skies/space3.png');
      this.load.image('logo', 'assets/sprites/phaser3-logo.png');
    }

    function create() {
      this.add.image(400, 300, 'sky');
      const logo = this.physics.add.image(400, 100, 'logo');
      logo.setVelocity(100, 200);
      logo.setBounce(1, 1);
      logo.setCollideWorldBounds(true);
    }
  } else {
    console.error('Phaser.js is not loaded');
  }
}
document.addEventListener('DOMContentLoaded', startPhaserGame);
