use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub label: String,
    pub centroid: Vec<f32>,
    pub member_indices: Vec<usize>,
}

const MAX_ITERATIONS: usize = 50;

pub fn cluster_energy_vectors(vectors: &[Vec<f32>]) -> Vec<Cluster> {
    if vectors.len() < 2 {
        return Vec::new();
    }

    let k = select_k(vectors.len());
    let dim = vectors[0].len();

    // K-means++ initialization
    let mut rng = rand::thread_rng();
    let mut centroids = kmeans_plus_plus_init(vectors, k, &mut rng);

    let mut assignments = vec![0usize; vectors.len()];

    for _ in 0..MAX_ITERATIONS {
        // Assign each vector to nearest centroid
        let mut changed = false;
        for (i, vec) in vectors.iter().enumerate() {
            let nearest = nearest_centroid(vec, &centroids);
            if nearest != assignments[i] {
                assignments[i] = nearest;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        // Update centroids
        let mut new_centroids = vec![vec![0.0f32; dim]; k];
        let mut counts = vec![0usize; k];

        for (i, vec) in vectors.iter().enumerate() {
            let c = assignments[i];
            counts[c] += 1;
            for (j, &v) in vec.iter().enumerate() {
                new_centroids[c][j] += v;
            }
        }

        for c in 0..k {
            if counts[c] > 0 {
                for j in 0..dim {
                    new_centroids[c][j] /= counts[c] as f32;
                }
            } else {
                // Keep old centroid for empty clusters
                new_centroids[c] = centroids[c].clone();
            }
        }

        centroids = new_centroids;
    }

    // Build clusters
    let mut clusters: Vec<Cluster> = (0..k)
        .map(|c| Cluster {
            label: format!("Cluster {}", c + 1),
            centroid: centroids[c].clone(),
            member_indices: Vec::new(),
        })
        .collect();

    for (i, &c) in assignments.iter().enumerate() {
        clusters[c].member_indices.push(i);
    }

    // Remove empty clusters
    clusters.retain(|c| !c.member_indices.is_empty());

    // Sort by average energy (low to high)
    clusters.sort_by(|a, b| {
        let avg_a = avg_energy(&a.centroid);
        let avg_b = avg_energy(&b.centroid);
        avg_a.partial_cmp(&avg_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Relabel after sorting
    for (i, cluster) in clusters.iter_mut().enumerate() {
        cluster.label = format!("Cluster {}", i + 1);
    }

    clusters
}

fn select_k(n: usize) -> usize {
    let k = ((n as f64) / 2.0).sqrt().round() as usize;
    k.clamp(2, 10)
}

fn kmeans_plus_plus_init(vectors: &[Vec<f32>], k: usize, rng: &mut impl Rng) -> Vec<Vec<f32>> {
    let mut centroids = Vec::with_capacity(k);

    // Pick first centroid randomly
    let first = rng.gen_range(0..vectors.len());
    centroids.push(vectors[first].clone());

    for _ in 1..k {
        // Compute distance from each point to nearest existing centroid
        let mut distances: Vec<f32> = vectors
            .iter()
            .map(|v| {
                centroids
                    .iter()
                    .map(|c| squared_distance(v, c))
                    .fold(f32::MAX, f32::min)
            })
            .collect();

        // Normalize to probability distribution
        let total: f32 = distances.iter().sum();
        if total < 1e-10 {
            // All points are identical, just pick randomly
            let idx = rng.gen_range(0..vectors.len());
            centroids.push(vectors[idx].clone());
            continue;
        }

        for d in &mut distances {
            *d /= total;
        }

        // Weighted random selection
        let threshold: f32 = rng.gen();
        let mut cumulative = 0.0f32;
        let mut selected = vectors.len() - 1;
        for (i, &d) in distances.iter().enumerate() {
            cumulative += d;
            if cumulative >= threshold {
                selected = i;
                break;
            }
        }

        centroids.push(vectors[selected].clone());
    }

    centroids
}

fn squared_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x - y) * (x - y))
        .sum()
}

fn nearest_centroid(point: &[f32], centroids: &[Vec<f32>]) -> usize {
    centroids
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            squared_distance(point, a)
                .partial_cmp(&squared_distance(point, b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn avg_energy(centroid: &[f32]) -> f32 {
    if centroid.is_empty() {
        return 0.0;
    }
    centroid.iter().sum::<f32>() / centroid.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_two_distinct_groups() {
        // Group A: low energy vectors
        // Group B: high energy vectors
        let mut vectors = Vec::new();
        for _ in 0..10 {
            vectors.push(vec![0.1; 128]);
        }
        for _ in 0..10 {
            vectors.push(vec![0.9; 128]);
        }

        let clusters = cluster_energy_vectors(&vectors);
        assert!(clusters.len() >= 2);

        // All 20 points should be assigned
        let total: usize = clusters.iter().map(|c| c.member_indices.len()).sum();
        assert_eq!(total, 20);
    }

    #[test]
    fn test_cluster_sorts_by_energy() {
        let mut vectors = Vec::new();
        for _ in 0..10 {
            vectors.push(vec![0.1; 128]);
        }
        for _ in 0..10 {
            vectors.push(vec![0.9; 128]);
        }

        let clusters = cluster_energy_vectors(&vectors);
        if clusters.len() >= 2 {
            let avg_first = avg_energy(&clusters[0].centroid);
            let avg_last = avg_energy(&clusters[clusters.len() - 1].centroid);
            assert!(avg_first <= avg_last);
        }
    }

    #[test]
    fn test_cluster_single_vector() {
        let vectors = vec![vec![0.5; 128]];
        let clusters = cluster_energy_vectors(&vectors);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_cluster_identical_vectors() {
        let vectors = vec![vec![0.5; 128]; 10];
        let clusters = cluster_energy_vectors(&vectors);
        // Should not panic, all points in one cluster
        let total: usize = clusters.iter().map(|c| c.member_indices.len()).sum();
        assert_eq!(total, 10);
    }

    #[test]
    fn test_k_selection() {
        assert_eq!(select_k(2), 2);   // sqrt(1) = 1, clamp to 2
        assert_eq!(select_k(8), 2);   // sqrt(4) = 2
        assert_eq!(select_k(50), 5);  // sqrt(25) = 5
        assert_eq!(select_k(200), 10); // sqrt(100) = 10
        assert_eq!(select_k(500), 10); // sqrt(250) > 10, clamp to 10
    }
}
