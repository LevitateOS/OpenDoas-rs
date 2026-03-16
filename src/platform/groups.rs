use pwd_grp;

pub fn current_group_names() -> Result<Vec<String>, String> {
    Ok(current_group_info(pwd_grp::getgid().into())?.0)
}

pub fn current_group_ids() -> Result<Vec<u32>, String> {
    Ok(current_group_info(pwd_grp::getgid().into())?.1)
}

pub fn current_group_info(primary_gid: u32) -> Result<(Vec<String>, Vec<u32>), String> {
    let mut group_ids: Vec<u32> = pwd_grp::getgroups()
        .map_err(|err| err.to_string())?
        .iter()
        .map(|group_id| (*group_id).into())
        .collect();
    if !group_ids.contains(&primary_gid) {
        group_ids.push(primary_gid);
    }

    let group_names = resolve_group_names(&group_ids)?;

    Ok((group_names, group_ids))
}

fn resolve_group_names(group_ids: &[u32]) -> Result<Vec<String>, String> {
    let mut group_names = Vec::new();
    for group_id in group_ids {
        let Some(group) = pwd_grp::getgrgid((*group_id).into()).map_err(|err| err.to_string())?
        else {
            continue;
        };
        group_names.push(group.name);
    }

    Ok(group_names)
}

#[cfg(test)]
mod tests {
    use super::resolve_group_names;

    #[test]
    fn ignores_unresolved_group_ids() {
        let group_names =
            resolve_group_names(&[u32::MAX]).expect("expected unresolved gid to be ignored");

        assert!(group_names.is_empty());
    }
}
