document.addEventListener('DOMContentLoaded', function() {
    // Header animation
    const header = document.querySelector('.header');
    header.style.animation = 'pulse 2s infinite';
    header.style.transformOrigin = '50% 50%';

    // Adding hover effects to nav links
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('mouseover', function() {
            this.style.transform = 'scale(1.1)';
            this.style.transition = 'transform 0.3s';
        });

        link.addEventListener('mouseout', function() {
            this.style.transform = 'scale(1)';
        });
    });

    // Animate unplayed songs with a bounce-in effect
    const unplayedSongs = document.querySelectorAll('.unplayed_song');
    unplayedSongs.forEach((song, index) => {
        song.style.animation = `bounceIn 1s ease ${index * 0.2}s`;
        song.style.opacity = '1';
    });

    // Image hover effect
    const aiSongImages = document.querySelectorAll('.ai_song_image img');
    aiSongImages.forEach(image => {
        image.addEventListener('mouseover', function() {
            this.style.filter = 'brightness(1.2)';
            this.style.transition = 'filter 0.3s';
        });

        image.addEventListener('mouseout', function() {
            this.style.filter = 'brightness(1)';
        });
    });

    // Animate videos on load
    const videos = document.querySelectorAll('.video video');
    videos.forEach((video, index) => {
        video.style.opacity = '0';
        video.style.animation = `fadeIn 1.5s ease-out ${index * 0.5}s forwards`;
    });

    // Add audio play animation
    const audioElements = document.querySelectorAll('.song_source audio');
    audioElements.forEach(audio => {
        audio.addEventListener('play', function() {
            this.parentElement.style.border = '2px solid #34ebb7';
            this.parentElement.style.transition = 'border 0.3s';
        });

        audio.addEventListener('pause', function() {
            this.parentElement.style.border = 'none';
        });
    });

    // Load CSS file dynamically
    const cssFile = document.createElement('link');
    cssFile.rel = 'stylesheet';
    cssFile.type = 'text/css';
    cssFile.href = '/static/styles.css';
    document.head.appendChild(cssFile);
});

// Keyframe animations
const styleSheet = document.styleSheets[0];
styleSheet.insertRule(`
@keyframes pulse {
    0% { transform: scale(1); }
    50% { transform: scale(1.05); }
    100% { transform: scale(1); }
}`, styleSheet.cssRules.length);

styleSheet.insertRule(`
@keyframes bounceIn {
    from {
        transform: translateY(-30px);
        opacity: 0;
    }
    to {
        transform: translateY(0);
        opacity: 1;
    }
}`, styleSheet.cssRules.length);

styleSheet.insertRule(`
@keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}`, styleSheet.cssRules.length);

styleSheet.insertRule(`
@keyframes bgGradient {
    0% {
        background-position: 0% 50%;
    }
    50% {
        background-position: 100% 50%;
    }
    100% {
        background-position: 0% 50%;
    }
}`, styleSheet.cssRules.length);

// Dynamic background gradient animation
styleSheet.insertRule(`
body {
    background: linear-gradient(45deg, #ff6b6b, #f0e130, #3651e6);
    background-size: 400% 400%;
    animation: bgGradient 15s ease infinite;
}`, styleSheet.cssRules.length);

// Containers fade-in animation
styleSheet.insertRule(`
@keyframes fadeInContainer {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
}`, styleSheet.cssRules.length);

const containers = document.querySelectorAll('.header-container, .nav-container, .unplayed_songs, .current-song, .grid-container, .users-container, .songs-container');
containers.forEach(container => {
    container.style.animation = 'fadeInContainer 2s ease-out';
});

// Hover effect for song links
const songLinks = document.querySelectorAll('.song_link a, .unplayed_song a');
songLinks.forEach(link => {
    link.addEventListener('mouseover', function() {
        this.style.color = '#ff6b6b';
        this.style.transition = 'color 0.3s ease';
    });
    link.addEventListener('mouseout', function() {
        this.style.color = '#0073e6';
    });
});

// Add bounce to page header
const pageHeader = document.querySelector('.header a');
pageHeader.style.animation = 'bounceHeader 2s infinite';