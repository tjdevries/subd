document.addEventListener('DOMContentLoaded', (event) => {
    // Animate the header text on load
    const header = document.querySelector('.header a');
    header.style.transition = 'transform 1s, color 1s';
    header.style.transform = 'scale(1.1)';
    header.style.color = '#ff6347';

    setTimeout(() => {
        header.style.transform = 'scale(1)';
        header.style.color = '';
    }, 1000);

    // Add hover effect to navigation links
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('mouseover', () => {
            link.style.transition = 'background-color 0.3s';
            link.style.backgroundColor = '#f0f8ff';
        });
        link.addEventListener('mouseout', () => {
            link.style.backgroundColor = '';
        });
    });

    // Animation effects for unplayed songs
    const unplayedSongs = document.querySelectorAll('.unplayed_song a');
    unplayedSongs.forEach(song => {
        song.addEventListener('mouseover', () => {
            song.style.transition = 'color 0.3s';
            song.style.color = '#32cd32';
        });
        song.addEventListener('mouseout', () => {
            song.style.color = '';
        });
    });

    // Dynamic score update for images
    const imageVotes = document.querySelectorAll('.image_voting');
    imageVotes.forEach(voteArea => {
        const votingOptions = voteArea.textContent.match(/!love\s(\d+)/g);
        if (votingOptions) {
            votingOptions.forEach(option => {
                const imageId = option.split(' ')[1];
                voteArea.style.cursor = 'pointer';

                voteArea.addEventListener('click', () => {
                    const votesDisplay = voteArea.nextElementSibling;
                    let currentVotes = parseInt(votesDisplay.textContent.split(': ')[1]);
                    votesDisplay.textContent = `Love: ${++currentVotes} | Hate: 0`;
                });
            });
        }
    });

    // Interactive elements on video
    const videos = document.querySelectorAll('video');
    videos.forEach(video => {
        video.addEventListener('mouseenter', () => {
            video.play();
            video.style.transition = 'transform 0.3s';
            video.style.transform = 'scale(1.1)';
        });

        video.addEventListener('mouseleave', () => {
            video.pause();
            video.style.transform = 'scale(1)';
        });
    });
});