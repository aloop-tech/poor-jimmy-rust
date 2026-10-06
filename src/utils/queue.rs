//! Queue edits by position. Positions are 1-based, as /list shows them, and
//! position 1 is the song playing now, which these functions never touch.

use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub enum QueueError {
    /// Position 1, the song playing now
    CurrentSong,
    /// No song at this position
    OutOfRange(usize),
}

impl QueueError {
    pub fn message(&self) -> String {
        match self {
            QueueError::CurrentSong => {
                "That's the song playing now! Use **/skip** to skip it.".to_string()
            }
            QueueError::OutOfRange(position) => format!(
                "There's no song at position {}! Use **/list** to see the queue.",
                position
            ),
        }
    }
}

/// Turn a 1-based position into an index, rejecting the current song
fn upcoming_index<T>(queue: &VecDeque<T>, position: usize) -> Result<usize, QueueError> {
    match position {
        1 => Err(QueueError::CurrentSong),
        _ if position == 0 || position > queue.len() => Err(QueueError::OutOfRange(position)),
        _ => Ok(position - 1),
    }
}

/// Remove and return the song at `position`
pub fn remove_upcoming<T>(queue: &mut VecDeque<T>, position: usize) -> Result<T, QueueError> {
    let index = upcoming_index(queue, position)?;
    Ok(queue.remove(index).expect("index checked above"))
}

/// Move the song at `from` so it ends up at position `to`
pub fn move_upcoming<T>(queue: &mut VecDeque<T>, from: usize, to: usize) -> Result<(), QueueError> {
    let from_index = upcoming_index(queue, from)?;
    let to_index = upcoming_index(queue, to)?;

    let song = queue.remove(from_index).expect("index checked above");
    queue.insert(to_index, song);
    Ok(())
}

/// Shuffle everything after the current song. Returns how many songs were shuffled.
pub fn shuffle_upcoming<T>(queue: &mut VecDeque<T>, rng: &mut fastrand::Rng) -> usize {
    let upcoming = queue.make_contiguous().get_mut(1..).unwrap_or_default();
    rng.shuffle(upcoming);
    upcoming.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue(len: usize) -> VecDeque<usize> {
        (1..=len).collect()
    }

    #[test]
    fn remove_takes_an_upcoming_song() {
        let mut q = queue(4);
        assert_eq!(remove_upcoming(&mut q, 3), Ok(3));
        assert_eq!(q, [1, 2, 4]);

        assert_eq!(remove_upcoming(&mut q, 3), Ok(4));
        assert_eq!(q, [1, 2]);
    }

    #[test]
    fn remove_rejects_the_current_song_and_bad_positions() {
        let mut q = queue(3);
        assert_eq!(remove_upcoming(&mut q, 1), Err(QueueError::CurrentSong));
        assert_eq!(remove_upcoming(&mut q, 0), Err(QueueError::OutOfRange(0)));
        assert_eq!(remove_upcoming(&mut q, 4), Err(QueueError::OutOfRange(4)));
        assert_eq!(q, [1, 2, 3]);

        let mut empty = queue(0);
        assert_eq!(
            remove_upcoming(&mut empty, 2),
            Err(QueueError::OutOfRange(2))
        );
    }

    #[test]
    fn move_reorders_upcoming_songs_in_both_directions() {
        let mut q = queue(5);
        move_upcoming(&mut q, 5, 2).unwrap();
        assert_eq!(q, [1, 5, 2, 3, 4]);

        move_upcoming(&mut q, 2, 5).unwrap();
        assert_eq!(q, [1, 2, 3, 4, 5]);

        move_upcoming(&mut q, 3, 3).unwrap();
        assert_eq!(q, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn move_rejects_the_current_song_and_bad_positions() {
        let mut q = queue(3);
        assert_eq!(move_upcoming(&mut q, 1, 3), Err(QueueError::CurrentSong));
        assert_eq!(move_upcoming(&mut q, 3, 1), Err(QueueError::CurrentSong));
        assert_eq!(move_upcoming(&mut q, 2, 9), Err(QueueError::OutOfRange(9)));
        assert_eq!(move_upcoming(&mut q, 9, 2), Err(QueueError::OutOfRange(9)));
        assert_eq!(q, [1, 2, 3]);
    }

    #[test]
    fn shuffle_keeps_the_current_song_first_and_every_song() {
        let mut rng = fastrand::Rng::with_seed(7);
        let mut q = queue(20);

        assert_eq!(shuffle_upcoming(&mut q, &mut rng), 19);
        assert_eq!(q[0], 1);

        let mut sorted: Vec<usize> = q.iter().copied().collect();
        sorted.sort();
        assert_eq!(sorted, (1..=20).collect::<Vec<_>>());
        assert_ne!(
            q,
            queue(20),
            "seeded shuffle of 19 songs should change the order"
        );
    }

    #[test]
    fn shuffle_handles_short_queues() {
        let mut rng = fastrand::Rng::with_seed(7);
        assert_eq!(shuffle_upcoming(&mut queue(0), &mut rng), 0);
        assert_eq!(shuffle_upcoming(&mut queue(1), &mut rng), 0);
        assert_eq!(shuffle_upcoming(&mut queue(2), &mut rng), 1);
    }
}
