extern crate curl;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate ajson;

use curl::easy::{Easy, List};
use std::time::Duration;
use std::thread;
use std::io::Read;
use serde_derive::{Serialize, Deserialize};

fn main() {
    //Required variables for updating DNS Record
    //change variables here to use this script
    //Account Email
    let login_email = "Your Cloudflare account email";
    //API Token
    //take extra care to keep this variable secret
    let global_api_key = "Your Cloudflare Global API Key";
    //Domain to change DNS Record
    let domain = "yourdomain.tld";
    
    loop {
        //Zone_id
        let mut url = String::from("https://api.cloudflare.com/client/v4/zones?name=");
        url.push_str(&domain);
        url.push_str("&status=active");

        //defining header
        let mut x_auth_email = String::from("X-Auth-Email: ");
        x_auth_email.push_str(&login_email);

        let mut x_auth_key = String::from("X-Auth-Key: ");
        x_auth_key.push_str(&global_api_key);

        let content_type = String::from("Content-Type: application/json");

        let mut list1 = List::new();
        list1.append(&x_auth_email).unwrap();
        list1.append(&x_auth_key).unwrap();
        list1.append(&content_type).unwrap();
        
        //uses HTTPGET method to get Zone_id
        let mut zoneidraw = Vec::new();
        let mut url1 = Easy::new();
        url1.url(&url).unwrap();
        url1.get(true).unwrap();
        url1.http_headers(list1).unwrap();
        
        {
            let mut transfer = url1.transfer();
            transfer.write_function(|data| {
                zoneidraw.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
            transfer.perform().unwrap()
        }
        
        let zone_id = String::from_utf8_lossy(&zoneidraw);
        //ajson to parse zone_id from result
        let zid = ajson::get(&zone_id,r#"result.0.id"#).unwrap();
        //println!("Zone_id is {}", zid.as_str().trim());
        
        //DNS_Record_id
        let mut url0 = String::from("https://api.cloudflare.com/client/v4/zones/");
        url0.push_str(&zid.as_str().trim());
        url0.push_str("/dns_records?type=A&name=");
        url0.push_str(&domain);

        let mut list2 = List::new();
        list2.append(&x_auth_email).unwrap();
        list2.append(&x_auth_key).unwrap();
        list2.append(&content_type).unwrap();
        
        let mut dnsrecordidraw = Vec::new();
        let mut url2 = Easy::new();
        url2.url(&url0).unwrap();
        url2.get(true).unwrap();
        url2.http_headers(list2).unwrap();
        
        {
            let mut transfer = url2.transfer();
            transfer.write_function(|data| {
                dnsrecordidraw.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
            transfer.perform().unwrap()
        }
        
        let dns_record_id = String::from_utf8_lossy(&dnsrecordidraw);
        let dip = ajson::get(&dns_record_id, r#"result.0.content"#).unwrap();
        let pip = dip.as_str().trim();
        //captures current ip address from https://checkip.amazonaws.com
        //into a local vector "cip"
        //Current ip setting
        let mut cip = Vec::new();
        let mut handle = Easy::new();
        handle.url("https://checkip.amazonaws.com").unwrap();
        {
            let mut transfer = handle.transfer();
            transfer.write_function(|data| {
                cip.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }

        let ip1 = String::from_utf8_lossy(&cip);
        println!("Current IP address is {:?}", &ip1.trim());
        println!("Record IP address is {}", &pip);

        //if current ip is diffrent with past ip updates DNS Record
        if &ip1.trim() == &pip {
            println!("There's no need to update");
        } else if &ip1.trim() != &pip {
            println!("Updating DNS Record");
            update_record(&login_email, &global_api_key, &domain);
        }

        //Updating A_Record in every 10 minutes
        thread::sleep(Duration::from_secs(600));
    }
}

fn update_record(login_email: &str, global_api_key: &str, domain: &str) {
    //Fetching current server ip address
    let mut ip = Vec::new();
    let mut handle = Easy::new();
    handle.url("https://checkip.amazonaws.com").unwrap();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            ip.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    let current_ip = String::from_utf8_lossy(&ip);

    //data = {"type":"A","name":"user-domain","content":"ip-address","ttl":"1","proxied":false};
    //name
    let mut n = String::new();
    n.push_str(&domain);
    //content
    let mut c = String::new();
    c.push_str(&current_ip.trim());

    //makes data format then converts it into u8
    #[derive(Serialize,Deserialize)]
    struct Data {
        r#type: String,
        name: String,
        content: String,
        ttl: u8,
        proxied: bool
    }

    let data = Data {
        r#type: String::from("A"), 
        name: n, 
        content: c, 
        ttl: 1, 
        proxied: false
    };
    let serialized_data = serde_json::to_string(&data).unwrap();
    //println!("JSON data is {}", serialized_data);

    let data_to_upload = &mut serialized_data.as_bytes();

    //defining header
    let mut x_auth_email = String::from("X-Auth-Email: ");
    x_auth_email.push_str(&login_email);

    let mut x_auth_key = String::from("X-Auth-Key: ");
    x_auth_key.push_str(&global_api_key);

    let content_type = String::from("Content-Type: application/json");

    //header list
    let mut list = List::new();
    list.append(&x_auth_email).unwrap();
    list.append(&x_auth_key).unwrap();
    list.append(&content_type).unwrap();
    
    //Zone_id
    let mut url = String::from("https://api.cloudflare.com/client/v4/zones?name=");
    url.push_str(&domain);
    url.push_str("&status=active");

    let mut list1 = List::new();
    list1.append(&x_auth_email).unwrap();
    list1.append(&x_auth_key).unwrap();
    list1.append(&content_type).unwrap();
    
    //uses HTTPGET method to get Zone_id
    let mut zoneidraw = Vec::new();
    let mut url1 = Easy::new();
    url1.url(&url).unwrap();
    url1.get(true).unwrap();
    url1.http_headers(list1).unwrap();
    
    {
        let mut transfer = url1.transfer();
        transfer.write_function(|data| {
            zoneidraw.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap()
    }
    
    let zone_id = String::from_utf8_lossy(&zoneidraw);
    //ajson to parse zone_id from result
    let zid = ajson::get(&zone_id,r#"result.0.id"#).unwrap();
    println!("Zone_id is {}", zid.as_str().trim());
    
    //DNS_Record_id
    let mut url0 = String::from("https://api.cloudflare.com/client/v4/zones/");
    url0.push_str(&zid.as_str().trim());
    url0.push_str("/dns_records?type=A&name=");
    url0.push_str(&domain);

    let mut list2 = List::new();
    list2.append(&x_auth_email).unwrap();
    list2.append(&x_auth_key).unwrap();
    list2.append(&content_type).unwrap();
    
    let mut dnsrecordidraw = Vec::new();
    let mut url2 = Easy::new();
    url2.url(&url0).unwrap();
    url2.get(true).unwrap();
    url2.http_headers(list2).unwrap();
    
    {
        let mut transfer = url2.transfer();
        transfer.write_function(|data| {
            dnsrecordidraw.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap()
    }
    
    let dns_record_id = String::from_utf8_lossy(&dnsrecordidraw);
    let did = ajson::get(&dns_record_id, r#"result.0.id"#).unwrap();
    
    println!("DNS_Record_id is {}", &did.as_str().trim());

    //Using cURL PUT to send required header&data to update DNS Record
    let mut api_url = String::from("https://api.cloudflare.com/client/v4/zones/");
    api_url.push_str(&zid.as_str().trim());
    api_url.push_str("/dns_records/");
    api_url.push_str(&did.as_str().trim());

    let mut res = Vec::new();
    let mut api = Easy::new();
    api.url(&api_url).unwrap();
    api.put(true).unwrap();
    api.http_headers(list).unwrap();
    api.upload(true).unwrap();
    api.in_filesize(data_to_upload.len() as u64).unwrap();

    {
        let mut transfer = api.transfer();
        transfer.read_function(|into| {
            Ok(data_to_upload.read(into).unwrap())
        }).unwrap();
        transfer.write_function(|data| {
            res.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    let result_json = String::from_utf8_lossy(&res);
    let result_done = ajson::get(&result_json, "success").unwrap();
    println!("Result is {}", &result_done.as_str());
}
