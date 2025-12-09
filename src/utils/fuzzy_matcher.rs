use super::command_history::CommandHistory;

/// Result of a fuzzy match with score
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub command: String,
    pub score: i32,
}

/// Performs fuzzy matching on commands with frequency-based ranking
pub struct FuzzyMatcher;

impl FuzzyMatcher {
    /// Finds fuzzy matches for the input query
    ///
    /// Returns up to `limit` matches, sorted by score (highest first)
    pub fn find_matches(
        query: &str,
        commands: &[String],
        history: &CommandHistory,
        limit: usize,
    ) -> Vec<FuzzyMatch> {
        if query.is_empty() {
            // Return most frequent commands when no query
            return history
                .get_frequent_commands()
                .into_iter()
                .take(limit)
                .map(|(cmd, freq)| FuzzyMatch {
                    command: cmd,
                    score: freq as i32 * 100, // High score for frequent commands
                })
                .collect();
        }

        let query_lower = query.to_lowercase();
        let mut matches = Vec::new();

        for command in commands {
            if let Some(score) = Self::calculate_match_score(&query_lower, command, history) {
                matches.push(FuzzyMatch {
                    command: command.clone(),
                    score,
                });
            }
        }

        // Sort by score (descending)
        matches.sort_by(|a, b| b.score.cmp(&a.score));
        matches.truncate(limit);
        matches
    }

    /// Calculates match score for a command (None if no match)
    ///
    /// Scoring factors:
    /// - Prefix match: +100 points
    /// - Exact match: +500 points
    /// - Fuzzy match: based on character positions
    /// - Frequency boost: +10 points per usage
    fn calculate_match_score(query: &str, command: &str, history: &CommandHistory) -> Option<i32> {
        let command_lower = command.to_lowercase();

        // Exact match
        if query == command_lower {
            let freq_boost = history.get_frequency(command) as i32 * 10;
            return Some(500 + freq_boost);
        }

        // Prefix match
        if command_lower.starts_with(query) {
            let freq_boost = history.get_frequency(command) as i32 * 10;
            return Some(100 + freq_boost);
        }

        // Fuzzy match (all characters in order)
        if let Some(fuzzy_score) = Self::fuzzy_score(query, &command_lower) {
            let freq_boost = history.get_frequency(command) as i32 * 10;
            return Some(fuzzy_score + freq_boost);
        }

        None
    }

    /// Calculates fuzzy match score if all query characters appear in order
    ///
    /// Examples:
    /// - "gst" matches "git status" (git has 'g', status has 'st')
    /// - "ls" matches "less"
    /// - "dc" matches "docker"
    fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
        let mut query_chars = query.chars();
        let mut current_char = query_chars.next()?;
        let mut last_match_pos = 0;
        let mut score = 50; // Base fuzzy match score

        for (pos, target_char) in target.chars().enumerate() {
            if target_char == current_char {
                // Bonus for consecutive matches
                if pos == last_match_pos + 1 {
                    score += 10;
                }

                // Penalty for gaps
                let gap = pos.saturating_sub(last_match_pos);
                score -= gap as i32;

                last_match_pos = pos;

                // Move to next query character
                if let Some(next) = query_chars.next() {
                    current_char = next;
                } else {
                    // All query characters matched!
                    return Some(score.max(1)); // Minimum score of 1
                }
            }
        }

        // Not all characters matched
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let score = FuzzyMatcher::fuzzy_score("ls", "ls");
        assert!(score.is_some());
        assert!(score.unwrap() > 0);
    }

    #[test]
    fn test_prefix_match() {
        let score = FuzzyMatcher::fuzzy_score("git", "github");
        assert!(score.is_some());
    }

    #[test]
    fn test_fuzzy_match() {
        let score = FuzzyMatcher::fuzzy_score("gst", "git");
        assert!(score.is_none()); // 'st' not in 'git'

        let score = FuzzyMatcher::fuzzy_score("dc", "docker");
        assert!(score.is_some()); // 'd' and 'c' are in 'docker'
    }

    #[test]
    fn test_no_match() {
        let score = FuzzyMatcher::fuzzy_score("xyz", "abc");
        assert!(score.is_none());
    }
}
