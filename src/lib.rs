use dbscan::QueryAccelerator;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Label {
    Undefined,
    Noise,
    Cluster { id: u32, prev: usize },
}

/// Returns (# of clusters, vector containing the label for each of the points)
pub fn dbscan_parents<const D: usize>(points: &[[f32; D]], radius: f32, min_pts: usize) -> (u32, Vec<Label>) {
    let mut label = vec![Label::Undefined; points.len()];

    let time = std::time::Instant::now();
    let accel = QueryAccelerator::new(points, radius);
    println!("Query build: {}s", time.elapsed().as_secs_f32());

    let time = std::time::Instant::now();
    let mut current_cluster = 0;

    // TODO: Don't iterate by point, iterate by query accel chunk (Hope for fewer cache misses)
    for point_idx in 0..points.len() {
        if label[point_idx] != Label::Undefined {
            continue;
        }

        let neighbors = accel
            .query_neighbors(points, point_idx)
            .map(|neigh_idx| (neigh_idx, point_idx))
            .collect::<Vec<(usize, usize)>>();

        if neighbors.len() < min_pts {
            label[point_idx] = Label::Noise;
            continue;
        }

        label[point_idx] = Label::Cluster {
            id: current_cluster,
            prev: point_idx,
        };

        let mut queue = neighbors;

        while let Some((neighbor_idx, neighbors_parent_idx)) = queue.pop() {
            if label[neighbor_idx] == Label::Noise {
                label[neighbor_idx] = Label::Cluster {
                    id: current_cluster,
                    prev: neighbors_parent_idx,
                };
            }

            if label[neighbor_idx] != Label::Undefined {
                continue;
            }

            label[neighbor_idx] = Label::Cluster {
                id: current_cluster,
                prev: neighbors_parent_idx,
            };

            let neighbors = || accel.query_neighbors(points, neighbor_idx);

            if neighbors().count() >= min_pts {
                let pair_with_parent_idx = |sub_neighbor_idx| (sub_neighbor_idx, neighbor_idx);
                queue.extend(neighbors().map(pair_with_parent_idx));
            }
        }

        current_cluster += 1;
    }

    println!("DBSCAN: {}s", time.elapsed().as_secs_f32());

    (current_cluster, label)
}
