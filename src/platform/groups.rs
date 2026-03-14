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

    let group_names = group_ids
        .iter()
        .map(|group_id| {
            pwd_grp::getgrgid((*group_id).into())
                .map_err(|err| err.to_string())?
                .ok_or_else(|| format!("Failed to retrieve group {}", group_id))
                .map(|group| group.name)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((group_names, group_ids))
}
