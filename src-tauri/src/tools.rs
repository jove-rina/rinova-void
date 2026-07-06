/// 托盘菜单工具列表，需与 `src/tools/registry.ts` 保持同步。
pub struct ToolDef {
    pub id: &'static str,
    pub name: &'static str,
    pub route: &'static str,
}

pub const TOOLS: &[ToolDef] = &[
    ToolDef {
        id: "clash-service",
        name: "Clash 订阅服务",
        route: "/tool/clash-service",
    },
    ToolDef {
        id: "color-picker",
        name: "取色器",
        route: "/tool/color-picker",
    },
];

pub fn route_for(id: &str) -> Option<&'static str> {
    TOOLS.iter().find(|t| t.id == id).map(|t| t.route)
}
