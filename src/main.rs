use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize)]
struct SaveFile {
    factions: Vec<Faction>,
    armies: Vec<Army>,
    states: Vec<State>,
}

#[derive(Serialize, Deserialize)]
struct Faction {
    name: String,
    alliances: Vec<String>,
    auras: Vec<(String, u32)>,
    wealth: i32,
    provision_rate: f32,
    reserve: u32,
    score: u32,
    pp: i32,
    pp_gain: i32,
    withdraw_rate: f32,
    min_withdraw_rate: f32,
    attack_modifier: f32,
    defense_modifier: f32,
}

#[derive(Serialize, Deserialize)]
struct Army {
    name: String,
    owner: String,
    location: String,
    size: u32,
}

#[derive(Serialize, Deserialize)]
struct Aura {
    name: String,
    duration: u32,
}

#[derive(Serialize, Deserialize)]
struct State {
    name: String,
    owner: String,
    population: u32,
    tax_rate: f32,
    max_tax_rate: f32,
    attack_modifier: f32,
    defense_modifier: f32,
}

fn main() {
    startup_screen();
    let mut save_file = SaveFile {
        factions: Vec::new(),
        armies: Vec::new(),
        states: Vec::new(),
    };
    while true {
        let mut input = String::new();
        let mut valid_input = false;
        io::stdin()
            .read_line(&mut input)
            .expect("无法读取输入，请重试。");
        // println!("{}", input);
        let args: Vec<&str> = input.trim().split(" ").collect();
        match args[0] {
            "end_turn" => {
                println!("结束回合...");
                for faction in save_file.factions.iter_mut() {
                    let mut total_tax = 0;
                    let mut total_pp_gain = 0;
                    println!("\n{}本回合税收情况:", faction.name);
                    for state in save_file.states.iter_mut() {
                        if faction.name == state.owner {
                            let tax = state.population as f32 * state.tax_rate;
                            faction.wealth += tax as i32;
                            total_tax += tax as i32;
                            println!("- {}以{}税率征税{}。", state.name, state.tax_rate, tax);
                            if state.tax_rate > state.max_tax_rate {
                                let overtax =
                                    ((((state.tax_rate - state.max_tax_rate) * 100.0) as i32 / 5)
                                        * 10) as i32;
                                total_pp_gain -= overtax;
                                println!("!!{}税率超过上限，{}政治点被扣除!!", state.name, overtax)
                            }
                        }
                    }
                    println!("总计征税{}", total_tax);
                    total_pp_gain += faction.pp_gain;
                    faction.pp += total_pp_gain;
                    println!(
                        "{}本回合自动获得{}政治点，当前政治点: {}。",
                        faction.name, total_pp_gain, faction.pp
                    );
                    if faction.pp < 5 {
                        println!(
                            "!!!政治点不足5点，{}无法执行国策，撤兵率下限提高10%",
                            faction.name
                        );
                        faction.min_withdraw_rate += 0.1;
                        if faction.min_withdraw_rate > 1.0 {
                            faction.min_withdraw_rate = 1.0;
                        }
                        if faction.withdraw_rate < faction.min_withdraw_rate {
                            faction.withdraw_rate = faction.min_withdraw_rate;
                        }
                    }
                    for army in save_file.armies.iter_mut() {
                        if army.owner == faction.name {
                            faction.wealth -= (army.size as f32 * faction.provision_rate) as i32;
                        }
                    }
                    println!("{}本回合结余: {}", faction.name, faction.wealth);
                    if faction.wealth < 0 {
                        println!("!!!{}已进入欠晌状态!!!", faction.name);
                    }
                    println!("{}光环效果更新:", faction.name);
                    let mut auras_to_remove: Vec<usize> = vec![];
                    for (index, aura) in faction.auras.iter_mut().enumerate() {
                        aura.1 -= 1;
                        if aura.1 == 0 {
                            println!("{}效果结束。", aura.0);
                            auras_to_remove.push(index);
                        } else {
                            println!("{}, 持续时间剩余{}回合。", aura.0, aura.1);
                        }
                    }
                    for index in auras_to_remove.iter() {
                        faction.auras.remove(*index);
                    }
                }
                //结算战斗

                let mut battles: Vec<(usize, usize)> = vec![];
                for (i, army1) in save_file.armies.iter().enumerate() {
                    for (j, army2) in save_file.armies.iter().enumerate() {
                        if j <= i {
                            continue;
                        }
                        if army1.location == army2.location && army1.owner != army2.owner {
                            if save_file
                                .factions
                                .iter()
                                .find(|x| x.name == army1.owner)
                                .unwrap()
                                .alliances
                                .contains(&army2.owner)
                            {
                                continue;
                            }
                            battles.push((i as usize, j as usize));
                        }
                    }
                }

                for (i, j) in battles {
                    let army = &mut save_file.armies;
                    let faction1 = save_file
                        .factions
                        .iter()
                        .find(|x| x.name == army[i].owner)
                        .unwrap();
                    let faction2 = save_file
                        .factions
                        .iter()
                        .find(|x| x.name == army[j].owner)
                        .unwrap();
                    let state = save_file
                        .states
                        .iter()
                        .find(|x| x.name == army[i].location)
                        .unwrap();
                    let mut army1_score = army[i].size as f32 * (1.0 - faction1.withdraw_rate);
                    let mut army2_score = army[j].size as f32 * (1.0 - faction2.withdraw_rate);
                    if state.owner == army[i].owner {
                        army1_score *= faction1.defense_modifier * state.defense_modifier;
                        army2_score *= faction2.attack_modifier * state.attack_modifier;
                    } else if state.owner == army[j].owner {
                        army1_score *= faction1.attack_modifier * state.attack_modifier;
                        army2_score *= faction2.defense_modifier * state.defense_modifier;
                    } else {
                        army1_score *= faction1.attack_modifier * state.attack_modifier;
                        army2_score *= faction2.attack_modifier * state.attack_modifier;
                    }

                    if army1_score > army2_score {
                        println!(
                            "\n{}之战：{}{}击败了{}{}。",
                            army[i].location,
                            faction1.name,
                            army[i].name,
                            faction2.name,
                            army[j].name
                        );
                        if state.owner == army[j].owner {
                            println!("{}占领了{}。", army[i].owner, army[i].location);
                            save_file
                                .states
                                .iter_mut()
                                .find(|x| x.name == army[i].location)
                                .unwrap()
                                .owner = army[i].owner.clone();
                        } else {
                            println!("{}保卫了{}。", army[i].owner, army[i].location);
                        }
                        army[i].size = (army[i].size as f32 * faction1.withdraw_rate as f32) as u32
                            + (army1_score - army2_score) as u32;
                        army[j].size = (army[j].size as f32 * faction2.withdraw_rate as f32) as u32;
                    } else {
                        println!(
                            "\n{}之战：{}{}击败了{}{}。",
                            army[j].location,
                            faction2.name,
                            army[j].name,
                            faction1.name,
                            army[i].name
                        );
                        if state.owner == army[i].owner {
                            println!("{}占领了{}。", army[j].owner, army[j].location);
                            save_file
                                .states
                                .iter_mut()
                                .find(|x| x.name == army[j].location)
                                .unwrap()
                                .owner = army[j].owner.clone();
                        } else {
                            println!("{}保卫了{}。", army[j].owner, army[j].location);
                        }
                        army[j].size = (army[j].size as f32 * faction2.withdraw_rate as f32) as u32
                            + (army2_score - army1_score) as u32;
                        army[i].size = (army[i].size as f32 * faction1.withdraw_rate as f32) as u32;
                    }
                    //结算空城占领
                    for state in save_file.states.iter_mut() {
                        let mut count = (0, vec![]);
                        for army in save_file.armies.iter() {
                            if state.name == army.location {
                                count.0 += 1;
                                count.1.push(army.owner.clone());
                            }
                        }
                        if count.0 == 1
                            && state.owner != count.1[0]
                            && !save_file
                                .factions
                                .iter()
                                .find(|x| x.name == state.owner)
                                .unwrap()
                                .alliances
                                .contains(&count.1[0])
                        {
                            println!("{}占领{}。", count.1[0], state.name);
                            state.owner = count.1[0].clone();
                        }
                    }
                }
            }
            "remove_aura" => {
                if args.len() < 3 {
                    println!("请指定要移除的势力和光环的名称。");
                    continue;
                }
                let aura_name = args[1];
                let faction_name = args[2];
                for faction in save_file.factions.iter_mut() {
                    if faction.name != faction_name {
                        continue;
                    }
                    let mut auras_to_remove: Vec<usize> = vec![];
                    for (index, aura) in faction.auras.iter_mut().enumerate() {
                        if aura.0 == aura_name {
                            valid_input = true;
                            println!("{}的{}效果已移除。", faction.name, aura_name);
                            auras_to_remove.push(index);
                        }
                    }
                    for index in auras_to_remove.iter() {
                        faction.auras.remove(*index);
                    }
                }
                if !valid_input {
                    println!("未找到指定的光环。");
                }
            }
            "army" => {
                if args.len() < 5 {
                    println!("请指定要操作的军队的名称，所有者和位置。");
                    continue;
                }
                match args[1] {
                    "create" => {
                        if args.len() < 6 {
                            println!("请指定要创建的军队的名称，所有者，位置和规模。");
                            continue;
                        }
                        let army_name = args[2];
                        let owner = args[3];
                        let location = args[4];
                        let size = args[5].parse::<u32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if owner == faction.name {
                                valid_input = true;
                                if faction.reserve >= size {
                                    faction.reserve -= size;
                                    save_file.armies.push(Army {
                                        name: army_name.to_string(),
                                        owner: owner.to_string(),
                                        location: location.to_string(),
                                        size,
                                    });
                                    println!(
                                        "创建{}{}在{}，人数为{}。",
                                        owner, army_name, location, size
                                    );
                                } else {
                                    println!("该势力没有足够的预备役,请重试。");
                                }
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                    }
                    "add" => {
                        let army_name = args[2];
                        let owner = args[3];
                        let size = args[4].parse::<u32>().unwrap();

                        for faction in save_file.factions.iter_mut() {
                            if owner == faction.name {
                                valid_input = true;
                                if faction.reserve >= size {
                                    faction.reserve -= size;
                                    for army in save_file.armies.iter_mut() {
                                        if army_name == army.name && owner == army.owner {
                                            army.size += size;
                                            println!("{}{}的人数增加{}。", owner, army_name, size);
                                        }
                                    }
                                } else {
                                    println!("该势力没有足够的预备役,请重试。");
                                }
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                    }
                    "split" => {
                        if args.len() < 6 {
                            println!("请指定要分开的军队的名称，所有者，规模和新军队的名称。");
                            continue;
                        }
                        let army_name = args[2];
                        let owner = args[3];
                        let size = args[4].parse::<u32>().unwrap();
                        let new_army_name = args[5];

                        let mut location = String::new();

                        for army in save_file.armies.iter_mut() {
                            if army_name == army.name && owner == army.owner {
                                valid_input = true;
                                if army.size >= size {
                                    army.size -= size;
                                    location = army.location.to_string();
                                    println!(
                                        "{}{}的人数减少至{}，在{}创建{}{},人数为{}。",
                                        owner,
                                        army_name,
                                        army.size,
                                        army.location,
                                        owner,
                                        new_army_name,
                                        size
                                    );
                                } else {
                                    println!("{}{}没有足够的人数,请重试。", owner, army_name);
                                }
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                        save_file.armies.push(Army {
                            name: new_army_name.to_string(),
                            owner: owner.to_string(),
                            location: location.to_string(),
                            size,
                        });
                    }
                    "move" => {
                        let army_name = args[2];
                        let owner = args[3];
                        let location = args[4];
                        for state in save_file.states.iter() {
                            if state.name == location {
                                for army in save_file.armies.iter_mut() {
                                    if army.name == army_name && army.owner == owner {
                                        valid_input = true;
                                        let prev_location = army.location.clone();
                                        army.location = location.to_string();
                                        println!(
                                            "{}: {} -> {}",
                                            army_name, prev_location, location
                                        );
                                        break;
                                    }
                                }
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                    }
                    "disband" => {
                        let army_name = args[2];
                        let owner = args[3];
                        for army in save_file.armies.iter() {
                            if army.name == army_name && army.owner == owner {
                                valid_input = true;
                                for faction in save_file.factions.iter_mut() {
                                    if faction.name == owner {
                                        faction.reserve += army.size;
                                        println!("{}{}已解散。", owner, army_name);
                                    }
                                }
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                    }
                    _ => {
                        println!("未知的军队操作。");
                    }
                }
            }
            "change_every_state" => {
                if args.len() < 4 {
                    println!("请指定要更改的势力，州或值。");
                    continue;
                }
                match args[1] {
                    "tax_rate" => {
                        let faction_name = args[2];
                        let tax_rate = args[3].parse::<f32>().unwrap();
                        for state in save_file.states.iter_mut() {
                            if faction_name == state.owner {
                                valid_input = true;
                                state.tax_rate += tax_rate;
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                        println!("{}所有州的税率增加{}", faction_name, tax_rate);
                    }
                    "max_tax_rate" => {
                        let max_tax_rate = args[2].parse::<f32>().unwrap();
                        let faction_name = args[2];
                        for state in save_file.states.iter_mut() {
                            if faction_name == state.owner {
                                valid_input = true;
                                state.max_tax_rate += max_tax_rate;
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                        println!("{}所有州的最大税率增加{}", faction_name, max_tax_rate);
                    }
                    "population" => {
                        let population = args[2].parse::<u32>().unwrap();
                        let faction_name = args[2];
                        for state in save_file.states.iter_mut() {
                            if faction_name == state.owner {
                                valid_input = true;
                                state.population += population;
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                        println!("{}所有州的人口增加{}", faction_name, population);
                    }
                    _ => {
                        println!("无法识别的对象，请重试。");
                    }
                }
            }
            "transfer" => {
                if args.len() < 5 {
                    println!("请指定要转移的双方，资源类型和数值。");
                    continue;
                }
                match args[1] {
                    "wealth" => {
                        let faction_name = args[2];
                        let target_faction_name = args[3];
                        let wealth = args[4].parse::<i32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction_name == faction.name {
                                valid_input = true;
                                faction.wealth -= wealth;
                            }
                            if target_faction_name == faction.name {
                                valid_input = true;
                                faction.wealth += wealth;
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                        println!(
                            "{}转移{}财富给{}",
                            faction_name, wealth, target_faction_name
                        );
                    }
                    "reserve" => {
                        let faction_name = args[2];
                        let target_faction_name = args[3];
                        let reserve = args[4].parse::<u32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction_name == faction.name {
                                valid_input = true;
                                faction.reserve -= reserve;
                            }
                            if target_faction_name == faction.name {
                                valid_input = true;
                                faction.reserve += reserve;
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                        println!(
                            "{}转移{}预备役给{}",
                            faction_name, reserve, target_faction_name
                        );
                    }
                    "pp" => {
                        let faction_name = args[2];
                        let target_faction_name = args[3];
                        let pp = args[4].parse::<i32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction_name == faction.name {
                                valid_input = true;
                                faction.pp -= pp;
                            }
                            if target_faction_name == faction.name {
                                valid_input = true;
                                faction.pp += pp;
                            }
                        }
                        if !valid_input {
                            println!("参数有误，请重试。");
                            continue;
                        }
                        println!("{}转移{}政治点给{}", faction_name, pp, target_faction_name);
                    }
                    _ => {
                        println!("参数错误，请重试。");
                    }
                }
            }
            "alliance" => {
                if args.len() < 3 {
                    println!("请指定要结盟的双方。");
                    continue;
                }
                let faction_name = args[1];
                let target_faction_name = args[2];
                for faction in save_file.factions.iter_mut() {
                    if faction_name == faction.name {
                        valid_input = true;
                        faction.alliances.push(target_faction_name.to_string());
                    }
                    if target_faction_name == faction.name {
                        valid_input = true;
                        faction.alliances.push(faction_name.to_string());
                    }
                }
                if !valid_input {
                    println!("参数有误，请重试。");
                    continue;
                }
                println!("{}和{}结盟", faction_name, target_faction_name);
            }
            "betray" => {
                if args.len() < 3 {
                    println!("请指定要背叛的双方。");
                    continue;
                }
                let faction_name = args[1];
                let target_faction_name = args[2];
                for faction in save_file.factions.iter_mut() {
                    if faction_name == faction.name {
                        valid_input = true;
                        faction.alliances.retain(|x| x != target_faction_name);
                    }
                    if target_faction_name == faction.name {
                        valid_input = true;
                        faction.alliances.retain(|x| x != faction_name);
                    }
                }
                if !valid_input {
                    println!("参数有误，请重试。");
                    continue;
                }
                println!("{}背叛了{}", faction_name, target_faction_name);
            }
            "set" => {
                if args.len() < 4 {
                    println!("请指定要设置的对象和值。");
                    continue;
                }
                match args[1] {
                    "tax_rate" => {
                        let state_name = args[2];
                        let tax_rate = args[3].parse::<f32>().unwrap();
                        for state in save_file.states.iter_mut() {
                            if state.name == state_name {
                                valid_input = true;
                                state.tax_rate = tax_rate;
                                println!("{}: 税率设置为{}", state_name, tax_rate);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到州，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "max_tax_rate" => {
                        let state_name = args[2];
                        let max_tax_rate = args[3].parse::<f32>().unwrap();
                        for state in save_file.states.iter_mut() {
                            if state.name == state_name {
                                valid_input = true;
                                state.max_tax_rate = max_tax_rate;
                                println!("{}: 最大税率设置为{}", state_name, max_tax_rate);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到州，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "aura" => {
                        if args.len() < 5 {
                            println!("请指定势力名称,光环和持续回合数。");
                            continue;
                        }
                        let faction_name = args[2];
                        let aura_name = args[3];
                        let aura_duration = args[4].parse::<u32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.auras.push((aura_name.to_string(), aura_duration));
                                println!(
                                    "{}: 光环{}已添加,持续{}回合。",
                                    faction_name, aura_name, aura_duration
                                );
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "population" => {
                        let state_name = args[2];
                        let population = args[3].parse::<u32>().unwrap();
                        for state in save_file.states.iter_mut() {
                            if state.name == state_name {
                                valid_input = true;
                                state.population = population;
                                println!("{}: 人口设置为{}", state_name, population);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到州，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "withdraw_rate" => {
                        let faction_name = args[2];
                        let withdraw_rate = args[3].parse::<f32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.withdraw_rate = withdraw_rate;
                                println!("{}: 撤退率设置为{}", faction_name, withdraw_rate);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "min_withdraw_rate" => {
                        let faction_name = args[2];
                        let min_withdraw_rate = args[3].parse::<f32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.min_withdraw_rate = min_withdraw_rate;
                                println!("{}: 最小撤退率设置为{}", faction_name, min_withdraw_rate);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "change_score" => {
                        let faction_name = args[2];
                        let score = args[3].parse::<u32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.score += score;
                                println!("{}: 分数增加{}", faction_name, score);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "change_pp" => {
                        let faction_name = args[2];
                        let pp = args[3].parse::<i32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.pp += pp;
                                println!("{}: 政治点增加{}", faction_name, pp);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "change_wealth" => {
                        let faction_name = args[2];
                        let wealth = args[3].parse::<i32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.wealth += wealth;
                                println!("{}: 财富增加{}", faction_name, wealth);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "change_reserve" => {
                        let faction_name = args[2];
                        let reserve = args[3].parse::<u32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.reserve += reserve;
                                println!("{}: 预备役增加{}", faction_name, reserve);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "change_pp_gain" => {
                        let faction_name = args[2];
                        let pp_gain = args[3].parse::<i32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.pp_gain += pp_gain;
                                println!("{}: 政治点增长增加{}", faction_name, pp_gain);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "owner" => {
                        let state_name = args[2];
                        let owner = args[3];
                        for state in save_file.states.iter_mut() {
                            if state.name == state_name {
                                valid_input = true;
                                state.owner = owner.to_string();
                                println!("{}: 所有者设置为{}", state_name, owner);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到州，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "state_attack_modifier" => {
                        let state_name = args[2];
                        let modifier = args[3].parse::<f32>().unwrap();
                        for state in save_file.states.iter_mut() {
                            if state.name == state_name {
                                valid_input = true;
                                state.attack_modifier = modifier;
                                println!("{}: 攻城修正设置为{}", state_name, modifier);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到州，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "state_defense_modifier" => {
                        let state_name = args[2];
                        let modifier = args[3].parse::<f32>().unwrap();
                        for state in save_file.states.iter_mut() {
                            if state.name == state_name {
                                valid_input = true;
                                state.defense_modifier = modifier;
                                println!("{}: 守城修正设置为{}", state_name, modifier);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到州，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "faction_attack_modifier" => {
                        let faction_name = args[2];
                        let modifier = args[3].parse::<f32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.attack_modifier = modifier;
                                println!("{}: 攻击力修正设置为{}", faction_name, modifier);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    "faction_defense_modifier" => {
                        let faction_name = args[2];
                        let modifier = args[3].parse::<f32>().unwrap();
                        for faction in save_file.factions.iter_mut() {
                            if faction.name == faction_name {
                                valid_input = true;
                                faction.defense_modifier = modifier;
                                println!("{}: 防御力修正设置为{}", faction_name, modifier);
                                break;
                            }
                        }
                        if !valid_input {
                            println!("无法找到势力，请重试。");
                            continue;
                        }
                        continue;
                    }
                    _ => {
                        println!("未知对象，请重试。");
                    }
                }
            }
            "export" => {
                if args.len() < 2 {
                    println!("请指定导出范围。");
                    continue;
                }
                match args[1] {
                    "all" => {
                        export_all_data(&save_file);
                    }
                    _ => {
                        export_selected_data(&save_file, args[1]);
                    }
                }
            }
            "save" => {
                if args.len() < 2 {
                    println!("请指定存档名称。");
                    continue;
                }
                let file_name = args[1];
                save_to_file(file_name, &save_file);
            }
            "load" => {
                if args.len() < 2 {
                    println!("请指定存档名称。");
                    continue;
                }
                let file_name = args[1];
                save_file = match load_save_file(file_name) {
                    Some(save_file) => save_file,
                    None => continue,
                };
            }
            "help" => {
                print_manual();
            }
            "?" => {
                print_manual();
            }
            "exit" => {
                println!("正在退出...");
                break;
            }
            _ => {
                println!("未知指令，请重试。输入'help'或'?'来查看可用指令。");
            }
        }
    }
}

fn load_save_file(savefile: &str) -> Option<SaveFile> {
    println!("载入存档...");
    let save_path = format!("./{}.toml", savefile);
    let file = match std::fs::read_to_string(save_path.clone()) {
        Ok(file) => file,
        Err(_) => {
            println!("无法读取存档，请检查文件是否存在。");
            return None;
        }
    };
    let save_file = toml::from_str(&file).unwrap();
    println!("存档{save_path}载入成功。");
    Some(save_file)
}

fn save_to_file(savefile: &str, save_file: &SaveFile) {
    println!("保存存档...");
    let save_path = format!("./{}.toml", savefile);
    let file = toml::to_string(&save_file).unwrap();
    match std::fs::write(save_path.clone(), file) {
        Ok(_) => println!("存档{save_path}保存成功。"),
        Err(_) => println!("无法保存存档。"),
    }
}

fn startup_screen() {
    println!("元和中兴辅助[命令行终端] v 1.0");
    println!("由Youxuan Zhao制作\n");
    println!("请输入'help'或'?'来查看可用指令。");
}

fn print_manual() {
    println!("------指令------");
    println!("exit - 退出(不自动保存！)");
    println!("load <存档名> - 加载存档");
    println!("save <存档名> - 保存存档(如果文件存在则覆盖，不存在则创建)");
    println!("end_turn - 结束回合，开始结算。");
    println!("export <all|势力名> - 打印全部数据或打印特定势力的数据");
    println!("set <tax_rate> <州名> <值> - 手动设置州的税率");
    println!("set <max_tax_rate> <州名> <值> - 手动设置州的最大税率");
    println!("set <owner> <州名> <势力名> - 手动设置州的所有者");
    println!("set <aura> <势力名> <光环效果名> <持续时间(永久则输入999)> - 为势力添加光环效果指示");
    println!("set <change_pp_gain> <势力名> <值> - 将势力的政治点增长增加<值>");
    println!("set <change_pp> <势力名> <值> - 将势力的政治点增加<值>");
    println!("set <change_score> <势力名> <值> - 将势力的分数增加<值>");
    println!("set <population> <州名> <值> - 手动设置州人口");
    println!("set <faction_attack_modifier> <势力名> <值> - 设置势力的攻击力修正(小数形式)");
    println!("set <faction_defense_modifier> <势力名> <值> - 设置势力的防御力修正(小数形式)");
    println!("set <state_attack_modifier> <州名> <值> - 设置州的攻城修正(小数形式)");
    println!("set <state_defense_modifier> <州名> <值> - 设置州的守城修正(小数形式)");
    println!("set <change_wealth> <势力名> <值> - 将势力的财富增加<值>");
    println!("set <change_reserve> <势力名> <值> - 将势力的预备役增加<值>");
    println!("alliance <势力A> <势力B> - 势力A与势力B结盟");
    println!("betray <势力A> <势力B> - 势力A背弃与势力B的结盟，势力A会承担政治点惩罚");
    println!("army create <军队名> <势力名> <地区> <规模> - 创建军队，需要自定义军队名称");
    println!("army add <军队名> <势力名> <规模> - 为军队补员");
    println!("army split <军队名> <势力名> <规模> <新军队名> - 将军队分出规模为<规模>的一只新军队");
    println!("army move <军队名> <势力名> <目的地> - 将军队移动到地区(没有做任何的相遇检测和距离检测,需要手动判断)");
    println!("army disband <军队名> <势力名> - 解散军队，将人员转移至预备役");
    println!(
        "transfer <wealth|reserve|pp> <势力A> <势力B> <值> - 将<值>点<资源类型>从势力A转移至势力B"
    );
    println!("change_every_state <tax_rate|max_tax_rate|population> <势力名> <值> - 将<势力名>的所有州的<属性>增加<值>");
}

fn export_all_data(save_file: &SaveFile) {
    println!("导出玩家数据...");
    for faction in save_file.factions.iter() {
        println!("{}: ", faction.name);
        println!("财富: {}", faction.wealth);
        println!("军饷率: {}:1", faction.provision_rate);
        println!("预备役: {}", faction.reserve);
        println!("分数: {}", faction.score);
        println!("政治点数: {}", faction.pp);
        println!("政治点数每回合获取：{}", faction.pp_gain);
        println!("撤退率: {}", faction.withdraw_rate);
        println!("最小撤退率: {}", faction.min_withdraw_rate);
        println!("攻击修正: {}x", faction.attack_modifier);
        println!("防御修正: {}x", faction.defense_modifier);
        println!("\n联盟：");
        for alliance in faction.alliances.iter() {
            println!("{}", alliance);
        }
        println!("\n光环：");
        for (aura_name, aura_duration) in faction.auras.iter() {
            println!("{},持续时间{}回合", aura_name, aura_duration);
        }
        println!("\n地区: ");
        for state in save_file.states.iter() {
            if state.owner == faction.name {
                println!("{}: ", state.name);
                println!("- 人口: {}", state.population);
                println!("- 税率: {}", state.tax_rate);
                println!("- 最大税率: {}", state.max_tax_rate);
                println!("- 攻城优势: {}x", state.attack_modifier);
                println!("- 守城优势: {}x", state.defense_modifier);
            }
        }
        println!("\n军队: ");
        for army in save_file.armies.iter() {
            if army.owner == faction.name {
                println!("{}: ", army.name);
                println!("- 位置: {}", army.location);
                println!("- 规模: {}", army.size);
            }
        }
    }
}

fn export_selected_data(save_file: &SaveFile, faction_name: &str) {
    println!("导出玩家数据...");
    for faction in save_file.factions.iter() {
        if faction.name != faction_name {
            continue;
        }
        println!("{}: ", faction.name);
        println!("财富: {}", faction.wealth);
        println!("军饷率: {}:1", faction.provision_rate);
        println!("预备役: {}", faction.reserve);
        println!("分数: {}", faction.score);
        println!("政治点数: {}", faction.pp);
        println!("政治点数每回合获取：{}", faction.pp_gain);
        println!("撤退率: {}", faction.withdraw_rate);
        println!("最小撤退率: {}", faction.min_withdraw_rate);
        println!("攻击修正: {}x", faction.attack_modifier);
        println!("防御修正: {}x", faction.defense_modifier);
        println!("\n联盟：");
        for alliance in faction.alliances.iter() {
            println!("{}", alliance);
        }
        println!("\n光环：");
        for (aura_name, aura_duration) in faction.auras.iter() {
            println!("{},持续时间{}回合", aura_name, aura_duration);
        }
        println!("\n地区: ");
        for state in save_file.states.iter() {
            if state.owner == faction.name {
                println!("{}: ", state.name);
                println!("- 人口: {}", state.population);
                println!("- 税率: {}", state.tax_rate);
                println!("- 最大税率: {}", state.max_tax_rate);
                println!("- 攻城优势: {}x", state.attack_modifier);
                println!("- 守城优势: {}x", state.defense_modifier);
            }
        }
        println!("\n军队");
        for army in save_file.armies.iter() {
            if army.owner == faction.name {
                println!("{}: ", army.name);
                println!("- 位置: {}", army.location);
                println!("- 规模: {}", army.size);
            }
        }
    }
}
