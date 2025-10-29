use serde_json::{json, Value};
use std::collections::HashMap;

use crate::app_config::{AppType, McpConfig, MultiAppConfig};
use crate::error::AppError;

/// 基础校验：允许 stdio/http；或省略 type（视为 stdio）。对应必填字段存在
fn validate_server_spec(spec: &Value) -> Result<(), AppError> {
    if !spec.is_object() {
        return Err(AppError::McpValidation(
            "MCP 服务器连接定义必须为 JSON 对象".into(),
        ));
    }
    let t_opt = spec.get("type").and_then(|x| x.as_str());
    // 支持两种：stdio/http；若缺省 type 则按 stdio 处理（与社区常见 .mcp.json 一致）
    let is_stdio = t_opt.map(|t| t == "stdio").unwrap_or(true);
    let is_http = t_opt.map(|t| t == "http").unwrap_or(false);

    if !(is_stdio || is_http) {
        return Err(AppError::McpValidation(
            "MCP 服务器 type 必须是 'stdio' 或 'http'（或省略表示 stdio）".into(),
        ));
    }

    if is_stdio {
        let cmd = spec.get("command").and_then(|x| x.as_str()).unwrap_or("");
        if cmd.trim().is_empty() {
            return Err(AppError::McpValidation(
                "stdio 类型的 MCP 服务器缺少 command 字段".into(),
            ));
        }
    }
    if is_http {
        let url = spec.get("url").and_then(|x| x.as_str()).unwrap_or("");
        if url.trim().is_empty() {
            return Err(AppError::McpValidation(
                "http 类型的 MCP 服务器缺少 url 字段".into(),
            ));
        }
    }
    Ok(())
}

fn validate_mcp_entry(entry: &Value) -> Result<(), AppError> {
    let obj = entry
        .as_object()
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目必须为 JSON 对象".into()))?;

    let server = obj
        .get("server")
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目缺少 server 字段".into()))?;
    validate_server_spec(server)?;

    for key in ["name", "description", "homepage", "docs"] {
        if let Some(val) = obj.get(key) {
            if !val.is_string() {
                return Err(AppError::McpValidation(format!(
                    "MCP 服务器 {} 必须为字符串",
                    key
                )));
            }
        }
    }

    if let Some(tags) = obj.get("tags") {
        let arr = tags
            .as_array()
            .ok_or_else(|| AppError::McpValidation("MCP 服务器 tags 必须为字符串数组".into()))?;
        if !arr.iter().all(|item| item.is_string()) {
            return Err(AppError::McpValidation(
                "MCP 服务器 tags 必须为字符串数组".into(),
            ));
        }
    }

    if let Some(enabled) = obj.get("enabled") {
        if !enabled.is_boolean() {
            return Err(AppError::McpValidation(
                "MCP 服务器 enabled 必须为布尔值".into(),
            ));
        }
    }

    Ok(())
}

fn normalize_server_keys(map: &mut HashMap<String, Value>) -> usize {
    let mut change_count = 0usize;
    let mut renames: Vec<(String, String)> = Vec::new();

    for (key_ref, value) in map.iter_mut() {
        let key = key_ref.clone();
        let Some(obj) = value.as_object_mut() else {
            continue;
        };

        let id_value = obj.get("id").cloned();

        let target_id: String;

        match id_value {
            Some(id_val) => match id_val.as_str() {
                Some(id_str) => {
                    let trimmed = id_str.trim();
                    if trimmed.is_empty() {
                        obj.insert("id".into(), json!(key.clone()));
                        change_count += 1;
                        target_id = key.clone();
                    } else {
                        if trimmed != id_str {
                            obj.insert("id".into(), json!(trimmed));
                            change_count += 1;
                        }
                        target_id = trimmed.to_string();
                    }
                }
                None => {
                    obj.insert("id".into(), json!(key.clone()));
                    change_count += 1;
                    target_id = key.clone();
                }
            },
            None => {
                obj.insert("id".into(), json!(key.clone()));
                change_count += 1;
                target_id = key.clone();
            }
        }

        if target_id != key {
            renames.push((key, target_id));
        }
    }

    for (old_key, new_key) in renames {
        if old_key == new_key {
            continue;
        }
        if map.contains_key(&new_key) {
            log::warn!(
                "MCP 条目 '{}' 的内部 id '{}' 与现有键冲突，回退为原键",
                old_key,
                new_key
            );
            if let Some(value) = map.get_mut(&old_key) {
                if let Some(obj) = value.as_object_mut() {
                    if obj
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s != old_key)
                        .unwrap_or(true)
                    {
                        obj.insert("id".into(), json!(old_key.clone()));
                        change_count += 1;
                    }
                }
            }
            continue;
        }
        if let Some(mut value) = map.remove(&old_key) {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("id".into(), json!(new_key.clone()));
            }
            log::info!("MCP 条目键名已自动修复: '{}' -> '{}'", old_key, new_key);
            map.insert(new_key, value);
            change_count += 1;
        }
    }

    change_count
}

pub fn normalize_servers_for(config: &mut MultiAppConfig, app: &AppType) -> usize {
    let servers = &mut config.mcp_for_mut(app).servers;
    normalize_server_keys(servers)
}

fn extract_server_spec(entry: &Value) -> Result<Value, AppError> {
    let obj = entry
        .as_object()
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目必须为 JSON 对象".into()))?;
    let server = obj
        .get("server")
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目缺少 server 字段".into()))?;

    if !server.is_object() {
        return Err(AppError::McpValidation(
            "MCP 服务器 server 字段必须为 JSON 对象".into(),
        ));
    }

    Ok(server.clone())
}

/// 返回已启用的 MCP 服务器（过滤 enabled==true）
fn collect_enabled_servers(cfg: &McpConfig) -> HashMap<String, Value> {
    let mut out = HashMap::new();
    for (id, entry) in cfg.servers.iter() {
        let enabled = entry
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !enabled {
            continue;
        }
        match extract_server_spec(entry) {
            Ok(spec) => {
                out.insert(id.clone(), spec);
            }
            Err(err) => {
                log::warn!("跳过无效的 MCP 条目 '{}': {}", id, err);
            }
        }
    }
    out
}

pub fn get_servers_snapshot_for(
    config: &mut MultiAppConfig,
    app: &AppType,
) -> (HashMap<String, Value>, usize) {
    let normalized = normalize_servers_for(config, app);
    let mut snapshot = config.mcp_for(app).servers.clone();
    snapshot.retain(|id, value| {
        let Some(obj) = value.as_object_mut() else {
            log::warn!("跳过无效的 MCP 条目 '{}': 必须为 JSON 对象", id);
            return false;
        };

        obj.entry(String::from("id")).or_insert(json!(id));

        match validate_mcp_entry(value) {
            Ok(()) => true,
            Err(err) => {
                log::error!("config.json 中存在无效的 MCP 条目 '{}': {}", id, err);
                false
            }
        }
    });
    (snapshot, normalized)
}

pub fn upsert_in_config_for(
    config: &mut MultiAppConfig,
    app: &AppType,
    id: &str,
    spec: Value,
) -> Result<bool, AppError> {
    if id.trim().is_empty() {
        return Err(AppError::InvalidInput("MCP 服务器 ID 不能为空".into()));
    }
    normalize_servers_for(config, app);
    validate_mcp_entry(&spec)?;

    let mut entry_obj = spec
        .as_object()
        .cloned()
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目必须为 JSON 对象".into()))?;
    if let Some(existing_id) = entry_obj.get("id") {
        let Some(existing_id_str) = existing_id.as_str() else {
            return Err(AppError::McpValidation("MCP 服务器 id 必须为字符串".into()));
        };
        if existing_id_str != id {
            return Err(AppError::McpValidation(format!(
                "MCP 服务器条目中的 id '{}' 与参数 id '{}' 不一致",
                existing_id_str, id
            )));
        }
    } else {
        entry_obj.insert(String::from("id"), json!(id));
    }

    let value = Value::Object(entry_obj);

    let servers = &mut config.mcp_for_mut(app).servers;
    let before = servers.get(id).cloned();
    servers.insert(id.to_string(), value);

    Ok(before.is_none())
}

pub fn delete_in_config_for(
    config: &mut MultiAppConfig,
    app: &AppType,
    id: &str,
) -> Result<bool, AppError> {
    if id.trim().is_empty() {
        return Err(AppError::InvalidInput("MCP 服务器 ID 不能为空".into()));
    }
    normalize_servers_for(config, app);
    let existed = config.mcp_for_mut(app).servers.remove(id).is_some();
    Ok(existed)
}

/// 设置启用状态（不执行落盘或文件同步）
pub fn set_enabled_flag_for(
    config: &mut MultiAppConfig,
    app: &AppType,
    id: &str,
    enabled: bool,
) -> Result<bool, AppError> {
    if id.trim().is_empty() {
        return Err(AppError::InvalidInput("MCP 服务器 ID 不能为空".into()));
    }
    normalize_servers_for(config, app);
    if let Some(spec) = config.mcp_for_mut(app).servers.get_mut(id) {
        // 写入 enabled 字段
        let mut obj = spec
            .as_object()
            .cloned()
            .ok_or_else(|| AppError::McpValidation("MCP 服务器定义必须为 JSON 对象".into()))?;
        obj.insert("enabled".into(), json!(enabled));
        *spec = Value::Object(obj);
    } else {
        // 若不存在则直接返回 false
        return Ok(false);
    }

    Ok(true)
}

/// 将 config.json 中 enabled==true 的项投影写入 ~/.claude.json
pub fn sync_enabled_to_claude(config: &MultiAppConfig) -> Result<(), AppError> {
    let enabled = collect_enabled_servers(&config.mcp.claude);
    crate::claude_mcp::set_mcp_servers_map(&enabled)
}

/// 从 ~/.claude.json 导入 mcpServers 到 config.json（设为 enabled=true）。
/// 已存在的项仅强制 enabled=true，不覆盖其他字段。
pub fn import_from_claude(config: &mut MultiAppConfig) -> Result<usize, AppError> {
    let text_opt = crate::claude_mcp::read_mcp_json()?;
    let Some(text) = text_opt else { return Ok(0) };
    let mut changed = normalize_servers_for(config, &AppType::Claude);
    let v: Value = serde_json::from_str(&text)
        .map_err(|e| AppError::McpValidation(format!("解析 ~/.claude.json 失败: {}", e)))?;
    let Some(map) = v.get("mcpServers").and_then(|x| x.as_object()) else {
        return Ok(changed);
    };

    for (id, spec) in map.iter() {
        // 校验目标 spec
        validate_server_spec(spec)?;

        let entry = config
            .mcp_for_mut(&AppType::Claude)
            .servers
            .entry(id.clone());
        use std::collections::hash_map::Entry;
        match entry {
            Entry::Vacant(vac) => {
                let mut obj = serde_json::Map::new();
                obj.insert(String::from("id"), json!(id));
                obj.insert(String::from("name"), json!(id));
                obj.insert(String::from("server"), spec.clone());
                obj.insert(String::from("enabled"), json!(true));
                vac.insert(Value::Object(obj));
                changed += 1;
            }
            Entry::Occupied(mut occ) => {
                let value = occ.get_mut();
                let Some(existing) = value.as_object_mut() else {
                    log::warn!("MCP 条目 '{}' 不是 JSON 对象，覆盖为导入数据", id);
                    let mut obj = serde_json::Map::new();
                    obj.insert(String::from("id"), json!(id));
                    obj.insert(String::from("name"), json!(id));
                    obj.insert(String::from("server"), spec.clone());
                    obj.insert(String::from("enabled"), json!(true));
                    occ.insert(Value::Object(obj));
                    changed += 1;
                    continue;
                };

                let mut modified = false;
                let prev_enabled = existing
                    .get("enabled")
                    .and_then(|b| b.as_bool())
                    .unwrap_or(false);
                if !prev_enabled {
                    existing.insert(String::from("enabled"), json!(true));
                    modified = true;
                }
                if existing.get("server").is_none() {
                    log::warn!("MCP 条目 '{}' 缺少 server 字段，覆盖为导入数据", id);
                    existing.insert(String::from("server"), spec.clone());
                    modified = true;
                }
                if existing.get("id").is_none() {
                    log::warn!("MCP 条目 '{}' 缺少 id 字段，自动填充", id);
                    existing.insert(String::from("id"), json!(id));
                    modified = true;
                }
                if modified {
                    changed += 1;
                }
            }
        }
    }
    Ok(changed)
}

/// 从 ~/.codex/config.toml 导入 MCP 到 config.json（Codex 作用域），并将导入项设为 enabled=true。
/// 支持两种 schema：[mcp.servers.<id>] 与 [mcp_servers.<id>]。
/// 已存在的项仅强制 enabled=true，不覆盖其他字段。
pub fn import_from_codex(config: &mut MultiAppConfig) -> Result<usize, AppError> {
    let text = crate::codex_config::read_and_validate_codex_config_text()?;
    if text.trim().is_empty() {
        return Ok(0);
    }
    let mut changed_total = normalize_servers_for(config, &AppType::Codex);

    let root: toml::Table = toml::from_str(&text)
        .map_err(|e| AppError::McpValidation(format!("解析 ~/.codex/config.toml 失败: {}", e)))?;

    // helper：处理一组 servers 表
    let mut import_servers_tbl = |servers_tbl: &toml::value::Table| {
        let mut changed = 0usize;
        for (id, entry_val) in servers_tbl.iter() {
            let Some(entry_tbl) = entry_val.as_table() else {
                continue;
            };

            // type 缺省为 stdio
            let typ = entry_tbl
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("stdio");

            // 构建 JSON 规范
            let mut spec = serde_json::Map::new();
            spec.insert("type".into(), json!(typ));

            match typ {
                "stdio" => {
                    if let Some(cmd) = entry_tbl.get("command").and_then(|v| v.as_str()) {
                        spec.insert("command".into(), json!(cmd));
                    }
                    if let Some(args) = entry_tbl.get("args").and_then(|v| v.as_array()) {
                        let arr = args
                            .iter()
                            .filter_map(|x| x.as_str())
                            .map(|s| json!(s))
                            .collect::<Vec<_>>();
                        if !arr.is_empty() {
                            spec.insert("args".into(), serde_json::Value::Array(arr));
                        }
                    }
                    if let Some(cwd) = entry_tbl.get("cwd").and_then(|v| v.as_str()) {
                        if !cwd.trim().is_empty() {
                            spec.insert("cwd".into(), json!(cwd));
                        }
                    }
                    if let Some(env_tbl) = entry_tbl.get("env").and_then(|v| v.as_table()) {
                        let mut env_json = serde_json::Map::new();
                        for (k, v) in env_tbl.iter() {
                            if let Some(sv) = v.as_str() {
                                env_json.insert(k.clone(), json!(sv));
                            }
                        }
                        if !env_json.is_empty() {
                            spec.insert("env".into(), serde_json::Value::Object(env_json));
                        }
                    }
                }
                "http" => {
                    if let Some(url) = entry_tbl.get("url").and_then(|v| v.as_str()) {
                        spec.insert("url".into(), json!(url));
                    }
                    if let Some(headers_tbl) = entry_tbl.get("headers").and_then(|v| v.as_table()) {
                        let mut headers_json = serde_json::Map::new();
                        for (k, v) in headers_tbl.iter() {
                            if let Some(sv) = v.as_str() {
                                headers_json.insert(k.clone(), json!(sv));
                            }
                        }
                        if !headers_json.is_empty() {
                            spec.insert("headers".into(), serde_json::Value::Object(headers_json));
                        }
                    }
                }
                _ => {}
            }

            let spec_v = serde_json::Value::Object(spec);

            // 校验
            if let Err(e) = validate_server_spec(&spec_v) {
                log::warn!("跳过无效 Codex MCP 项 '{}': {}", id, e);
                continue;
            }

            // 合并：仅强制 enabled=true
            use std::collections::hash_map::Entry;
            let entry = config
                .mcp_for_mut(&AppType::Codex)
                .servers
                .entry(id.clone());
            match entry {
                Entry::Vacant(vac) => {
                    let mut obj = serde_json::Map::new();
                    obj.insert(String::from("id"), json!(id));
                    obj.insert(String::from("name"), json!(id));
                    obj.insert(String::from("server"), spec_v.clone());
                    obj.insert(String::from("enabled"), json!(true));
                    vac.insert(serde_json::Value::Object(obj));
                    changed += 1;
                }
                Entry::Occupied(mut occ) => {
                    let value = occ.get_mut();
                    let Some(existing) = value.as_object_mut() else {
                        log::warn!("MCP 条目 '{}' 不是 JSON 对象，覆盖为导入数据", id);
                        let mut obj = serde_json::Map::new();
                        obj.insert(String::from("id"), json!(id));
                        obj.insert(String::from("name"), json!(id));
                        obj.insert(String::from("server"), spec_v.clone());
                        obj.insert(String::from("enabled"), json!(true));
                        occ.insert(serde_json::Value::Object(obj));
                        changed += 1;
                        continue;
                    };

                    let mut modified = false;
                    let prev = existing
                        .get("enabled")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(false);
                    if !prev {
                        existing.insert(String::from("enabled"), json!(true));
                        modified = true;
                    }
                    if existing.get("server").is_none() {
                        log::warn!("MCP 条目 '{}' 缺少 server 字段，覆盖为导入数据", id);
                        existing.insert(String::from("server"), spec_v.clone());
                        modified = true;
                    }
                    if existing.get("id").is_none() {
                        log::warn!("MCP 条目 '{}' 缺少 id 字段，自动填充", id);
                        existing.insert(String::from("id"), json!(id));
                        modified = true;
                    }
                    if modified {
                        changed += 1;
                    }
                }
            }
        }
        changed
    };

    // 1) 处理 mcp.servers
    if let Some(mcp_val) = root.get("mcp") {
        if let Some(mcp_tbl) = mcp_val.as_table() {
            if let Some(servers_val) = mcp_tbl.get("servers") {
                if let Some(servers_tbl) = servers_val.as_table() {
                    changed_total += import_servers_tbl(servers_tbl);
                }
            }
        }
    }

    // 2) 处理 mcp_servers
    if let Some(servers_val) = root.get("mcp_servers") {
        if let Some(servers_tbl) = servers_val.as_table() {
            changed_total += import_servers_tbl(servers_tbl);
        }
    }

    Ok(changed_total)
}

/// 将 config.json 中 Codex 的 enabled==true 项以 TOML 形式写入 ~/.codex/config.toml 的 [mcp.servers]
/// 策略：
/// - 读取现有 config.toml；若语法无效则报错，不尝试覆盖
/// - 仅更新 `mcp.servers` 或 `mcp_servers` 子表，保留 `mcp` 其它键
/// - 仅写入启用项；无启用项时清理对应子表
pub fn sync_enabled_to_codex(config: &MultiAppConfig) -> Result<(), AppError> {
    use toml_edit::{DocumentMut, Item, Table};

    // 1) 收集启用项（Codex 维度）
    let enabled = collect_enabled_servers(&config.mcp.codex);

    // 2) 读取现有 config.toml 文本；保持无效 TOML 的错误返回（不覆盖文件）
    let base_text = crate::codex_config::read_and_validate_codex_config_text()?;

    // 3) 使用 toml_edit 解析（允许空文件）
    let mut doc: DocumentMut = if base_text.trim().is_empty() {
        DocumentMut::default()
    } else {
        base_text
            .parse::<DocumentMut>()
            .map_err(|e| AppError::McpValidation(format!("解析 config.toml 失败: {}", e)))?
    };

    enum Target {
        McpServers,    // 顶层 mcp_servers
        McpDotServers, // mcp.servers
    }

    // 4) 选择目标风格：优先沿用既有子表；其次在 mcp 表下新建；最后退回顶层 mcp_servers
    let has_mcp_dot_servers = doc
        .get("mcp")
        .and_then(|m| m.get("servers"))
        .and_then(|s| s.as_table_like())
        .is_some();
    let has_mcp_servers = doc
        .get("mcp_servers")
        .and_then(|s| s.as_table_like())
        .is_some();
    let mcp_is_table = doc.get("mcp").and_then(|m| m.as_table_like()).is_some();

    let target = if has_mcp_dot_servers {
        Target::McpDotServers
    } else if has_mcp_servers {
        Target::McpServers
    } else if mcp_is_table {
        Target::McpDotServers
    } else {
        Target::McpServers
    };

    // 构造目标 servers 表（稳定的键顺序）
    let build_servers_table = || -> Table {
        let mut servers = Table::new();
        let mut ids: Vec<_> = enabled.keys().cloned().collect();
        ids.sort();
        for id in ids {
            let spec = enabled.get(&id).expect("spec must exist");
            let mut t = Table::new();
            let typ = spec.get("type").and_then(|v| v.as_str()).unwrap_or("stdio");
            t["type"] = toml_edit::value(typ);
            match typ {
                "stdio" => {
                    let cmd = spec.get("command").and_then(|v| v.as_str()).unwrap_or("");
                    t["command"] = toml_edit::value(cmd);
                    if let Some(args) = spec.get("args").and_then(|v| v.as_array()) {
                        let mut arr_v = toml_edit::Array::default();
                        for a in args.iter().filter_map(|x| x.as_str()) {
                            arr_v.push(a);
                        }
                        if !arr_v.is_empty() {
                            t["args"] = toml_edit::Item::Value(toml_edit::Value::Array(arr_v));
                        }
                    }
                    if let Some(cwd) = spec.get("cwd").and_then(|v| v.as_str()) {
                        if !cwd.trim().is_empty() {
                            t["cwd"] = toml_edit::value(cwd);
                        }
                    }
                    if let Some(env) = spec.get("env").and_then(|v| v.as_object()) {
                        let mut env_tbl = Table::new();
                        for (k, v) in env.iter() {
                            if let Some(s) = v.as_str() {
                                env_tbl[&k[..]] = toml_edit::value(s);
                            }
                        }
                        if !env_tbl.is_empty() {
                            t["env"] = Item::Table(env_tbl);
                        }
                    }
                }
                "http" => {
                    let url = spec.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    t["url"] = toml_edit::value(url);
                    if let Some(headers) = spec.get("headers").and_then(|v| v.as_object()) {
                        let mut h_tbl = Table::new();
                        for (k, v) in headers.iter() {
                            if let Some(s) = v.as_str() {
                                h_tbl[&k[..]] = toml_edit::value(s);
                            }
                        }
                        if !h_tbl.is_empty() {
                            t["headers"] = Item::Table(h_tbl);
                        }
                    }
                }
                _ => {}
            }
            servers[&id[..]] = Item::Table(t);
        }
        servers
    };

    // 5) 应用更新：仅就地更新目标子表；避免改动其它键/注释/空白
    if enabled.is_empty() {
        // 无启用项：移除两种 servers 表（如果存在），但保留 mcp 其它字段
        if let Some(mcp_item) = doc.get_mut("mcp") {
            if let Some(tbl) = mcp_item.as_table_like_mut() {
                tbl.remove("servers");
            }
        }
        doc.as_table_mut().remove("mcp_servers");
    } else {
        let servers_tbl = build_servers_table();
        match target {
            Target::McpDotServers => {
                // 确保 mcp 为表
                if doc.get("mcp").and_then(|m| m.as_table_like()).is_none() {
                    doc["mcp"] = Item::Table(Table::new());
                }
                doc["mcp"]["servers"] = Item::Table(servers_tbl);
                // 去重：若存在顶层 mcp_servers，则移除以避免重复定义
                doc.as_table_mut().remove("mcp_servers");
            }
            Target::McpServers => {
                doc["mcp_servers"] = Item::Table(servers_tbl);
                // 去重：若存在 mcp.servers，则移除该子表，保留 mcp 其它键
                if let Some(mcp_item) = doc.get_mut("mcp") {
                    if let Some(tbl) = mcp_item.as_table_like_mut() {
                        tbl.remove("servers");
                    }
                }
            }
        }
    }

    // 6) 写回（仅改 TOML，不触碰 auth.json）；toml_edit 会尽量保留未改区域的注释/空白/顺序
    let new_text = doc.to_string();
    let path = crate::codex_config::get_codex_config_path();
    crate::config::write_text_file(&path, &new_text)?;
    Ok(())
}
