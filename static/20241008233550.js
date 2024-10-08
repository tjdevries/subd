// JavaScript Animation for making the webpage interactive and fun

// Function to add a ripple effect on buttons
function createRipple(event) {
    const button = event.currentTarget;

    const circle = document.createElement("span");
    const diameter = Math.max(button.clientWidth, button.clientHeight);
    const radius = diameter / 2;

    circle.style.width = circle.style.height = `${diameter}px`;
    circle.style.left = `${event.clientX - button.offsetLeft - radius}px`;
    circle.style.top = `${event.clientY - button.offsetTop - radius}px`;
    circle.classList.add("ripple");

    const ripple = button.getElementsByClassName("ripple")[0];

    if (ripple) {
        ripple.remove();
    }

    button.appendChild(circle);
}

// Apply ripple effect to all nav-links
const navLinks = document.querySelectorAll('.nav-link');
navLinks.forEach(link => {
    link.addEventListener('click', createRipple);
});

// Animation for images in the image_scores section
const images = document.querySelectorAll('.ai_song_image img');
images.forEach(image => {
    image.addEventListener('mouseover', () => {
        image.style.transform = 'scale(1.1)';
        image.style.transition = 'transform 0.2s';
    });

    image.addEventListener('mouseout', () => {
        image.style.transform = 'scale(1)';
    });
});

// Add breadcrumbs for navigation
function createBreadcrumbs() {
    const breadcrumbs = document.createElement('div');
    breadcrumbs.className = 'breadcrumbs';
    breadcrumbs.innerHTML = `<a href="/">Home</a> / <span>Subpage</span>`;
    document.body.prepend(breadcrumbs);
}

createBreadcrumbs();

// Light up effects for the song playlist
const songLinks = document.querySelectorAll('.unplayed_song a');
songLinks.forEach(songLink => {
    songLink.addEventListener('mouseover', () => {
        songLink.style.color = 'gold';
        songLink.style.textShadow = '0 0 10px #FFD700';
        songLink.style.transition = 'color 0.3s, text-shadow 0.3s';
    });
    songLink.addEventListener('mouseout', () => {
        songLink.style.color = '';
        songLink.style.textShadow = '';
    });
});