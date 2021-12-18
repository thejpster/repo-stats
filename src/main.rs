use git2::Commit;
use chrono::TimeZone;
use git2::Repository;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    let mut collection = Vec::new();
    for repo_name in args {
        let repo_path = std::path::Path::new("./").join(&repo_name);
        let repo = Repository::open(&repo_path).or_else(|_| {
            eprintln!("{} not found in {:?}, cloning...", repo_name, repo_path);
            let url = format!("https://github.com/rust-embedded/{}.git", repo_name);
            Repository::clone(&url, &repo_path)
        }).unwrap();

        eprintln!("Opened {:?}", repo.path());

        let head = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
        let head_commit = head.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))?;

        display_commit(&head_commit);

        let mut revwalk = repo.revwalk()?;
        revwalk.push(head_commit.id())?;

        let mut oldest = None;
        let mut commit_count = 0;
        for oid in revwalk {
            let commit = repo.find_commit(oid?)?;
            let tm = commit_timestamp(&commit);
            if tm >= chrono::Utc.ymd(2021, 1, 1).and_hms(0,0,0) {
                oldest = Some(commit);
                commit_count += 1;
            } else {
                break;
            }
        }

        if let Some(oldest) = oldest {
            display_commit(&oldest);

            let tree = repo.find_tree(oldest.tree_id())?;

            let diff = repo.diff_tree_to_workdir(Some(&tree), None)?;

            let stats = diff.stats()?;

            collection.push((repo_name, commit_count, stats));
        }
    }

    collection.sort_by(|x, y| y.1.cmp(&x.1));

    for stat in collection {
        println!("### {}", stat.0);
        println!();
        println!(" * Commits {}", stat.1);
        println!(" * Files changed: {}", stat.2.files_changed());
        println!(" * Insertions: {}", stat.2.insertions());
        println!(" * Deletions: {}", stat.2.deletions());
        println!();
    }

    // Print the stats

    Ok(())
}

fn commit_timestamp(commit: &Commit) -> chrono::DateTime<chrono::Utc> {
    let timestamp = commit.time();
    let offset = chrono::offset::FixedOffset::east(timestamp.offset_minutes());
    offset.timestamp(timestamp.seconds(), 0).into()
}

fn display_commit(commit: &Commit) {
    let tm = commit_timestamp(commit);
    eprintln!("commit {}\nAuthor: {}\nDate:   {}",
             commit.id(),
             commit.author(),
             tm.to_rfc2822());
}

