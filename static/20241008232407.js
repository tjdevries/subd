document.addEventListener('DOMContentLoaded', function() {
    // Header animation
    const header = document.querySelector('.header-container');
    header.style.transition = 'transform 0.5s, opacity 0.5s';
    header.style.transform = 'translateY(-50px)';
    header.style.opacity = '0';
    window.setTimeout(() => {
        header.style.transform = 'translateY(0)';
        header.style.opacity = '1';
    }, 200);

    // Nav animation
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach((link, index) => {
        link.style.transition = `transform 0.3s ${index * 0.1}s ease-in-out`;
        link.style.transform = 'translateX(-20px)';
        link.style.opacity = '0';
        window.setTimeout(() => {
            link.style.transform = 'translateX(0)';
            link.style.opacity = '1';
        }, 500);
    });

    // Song hover effect
    const songs = document.querySelectorAll('.unplayed_song a, .song-link a');
    songs.forEach(song => {
        song.style.transition = 'transform 0.3s, color 0.3s';
        song.addEventListener('mouseover', () => {
            song.style.transform = 'scale(1.05)';
            song.style.color = '#ff4081';
        });
        song.addEventListener('mouseout', () => {
            song.style.transform = 'scale(1)';
            song.style.color = '#000000';
        });
    });

    // Image hover effect
    const imageElements = document.querySelectorAll('.ai_song_image img');
    imageElements.forEach(image => {
        image.style.transition = 'transform 0.3s';
        image.addEventListener('mouseover', () => {
            image.style.transform = 'rotate(5deg)';
        });
        image.addEventListener('mouseout', () => {
            image.style.transform = 'rotate(0deg)';
        });
    });
    
    // Animate videos
    const videos = document.querySelectorAll('video');
    videos.forEach(video => {
        video.style.transition = 'transform 0.5s';
        video.addEventListener('mouseover', () => {
            video.style.transform = 'scale(1.1)';
        });
        video.addEventListener('mouseout', () => {
            video.style.transform = 'scale(1)';
        });
    });

    // Footer disclaimer
    const footer = document.createElement('div');
    footer.innerText = "Enjoy the interactive music experience!";
    footer.style.position = 'fixed';
    footer.style.bottom = '10px';
    footer.style.width = '100%';
    footer.style.textAlign = 'center';
    footer.style.fontSize = '1.2em';
    footer.style.color = '#ffffff';
    footer.style.transition = 'opacity 0.5s';
    footer.style.opacity = '0';
    footer.style.backgroundColor = 'rgba(0, 0, 0, 0.7)';
    document.body.appendChild(footer);
    window.setTimeout(() => {
        footer.style.opacity = '1';
    }, 1000);
});
