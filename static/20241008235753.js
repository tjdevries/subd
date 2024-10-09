// Use this JS code for a creative, interactive web page inspired by the "Bagels of Dreams" theme
// Make sure to include this JavaScript in your HTML and link Three.js, Phaser.js, or any other desired libraries

document.addEventListener('DOMContentLoaded', function() {
    // Initialize the 3D scene using Three.js
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    // Add ambient light
    const ambientLight = new THREE.AmbientLight(0x404040);
    scene.add(ambientLight);

    // Bagel model
    const geometry = new THREE.TorusGeometry(1, 0.3, 16, 100);
    const material = new THREE.MeshStandardMaterial({ color: 0xf5deb3 });
    const bagel = new THREE.Mesh(geometry, material);
    scene.add(bagel);

    // Cheese glow effect
    const cheeseLight = new THREE.PointLight(0xfff700, 1, 100);
    cheeseLight.position.set(5, 5, 5);
    scene.add(cheeseLight);

    // Oven effect - rotate and hover
    function animate() {
        requestAnimationFrame(animate);

        bagel.rotation.x += 0.01;
        bagel.rotation.y += 0.01;

        cheeseLight.position.x = 5 * Math.sin(Date.now() * 0.001);
        cheeseLight.position.y = 5 * Math.cos(Date.now() * 0.001);

        renderer.render(scene, camera);
    }
    animate();

    camera.position.z = 5;

    // Add a Phaser.js animation for vibe
    const config = {
        type: Phaser.AUTO,
        width: 800,
        height: 600,
        scene: {
            create: createPhaserScene,
            update: updatePhaserScene
        }
    };

    const phaserGame = new Phaser.Game(config);

    function createPhaserScene() {
        this.cameras.main.setBackgroundColor('#ffcc00'); // Looks like cheesy bread delight
        this.add.text(100, 300, 'Pizza Bagels in the Air!', {
            font: '48px Arial',
            fill: '#ffffff'
        });

        // Add simple animation - perhaps bagels bouncing across the top
        const bagelSprite = this.physics.add.image(400, 50, 'bagel');
        bagelSprite.setVelocity(200, 0);
        bagelSprite.setBounce(1);
        bagelSprite.setCollideWorldBounds(true);
    }

    function updatePhaserScene() {
        // Additional game logic, animations, events
    }

    // Add more interactivity, allowing users to click or hover over animations for surprises
    const songLinks = document.querySelectorAll('.song_link a');
    songLinks.forEach(link => {
        link.addEventListener('mouseenter', () => {
            link.style.color = '#ff4500'; // Hover effect like "hot pizza"
        });
        link.addEventListener('mouseleave', () => {
            link.style.color = '';
        });
    });
});