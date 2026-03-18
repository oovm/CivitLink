//! MIME 消息构建模块
//!
//! 实现符合 RFC 5322 和 RFC 2047 标准的邮件消息构建功能。
//! 支持纯文本邮件、HTML 邮件、附件以及 multipart 消息格式。

use std::fmt;

use base64::{Engine as _, engine::general_purpose::STANDARD};

/// 邮件附件结构体
#[derive(Debug, Clone)]
pub struct Attachment {
    /// 附件文件名
    pub filename: String,
    /// 附件 MIME 类型
    pub content_type: String,
    /// 附件内容（二进制数据）
    pub data: Vec<u8>,
}

impl Attachment {
    /// 创建新的附件
    ///
    /// # 参数
    /// - `filename`: 附件文件名
    /// - `content_type`: MIME 类型，如 "application/pdf"
    /// - `data`: 附件的二进制内容
    pub fn new(filename: impl Into<String>, content_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self { filename: filename.into(), content_type: content_type.into(), data }
    }

    /// 从文本创建附件
    ///
    /// # 参数
    /// - `filename`: 附件文件名
    /// - `content`: 文本内容
    pub fn from_text(filename: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            content_type: "text/plain; charset=utf-8".to_string(),
            data: content.into().into_bytes(),
        }
    }
}

/// 邮件消息结构体
///
/// 存储邮件的所有元数据和内容，支持生成符合 RFC 5322 标准的邮件内容。
#[derive(Debug, Clone, Default)]
pub struct EmailMessage {
    /// 发件人地址
    pub from: String,
    /// 收件人地址列表
    pub to: Vec<String>,
    /// 抄送地址列表
    pub cc: Vec<String>,
    /// 密送地址列表
    pub bcc: Vec<String>,
    /// 邮件主题
    pub subject: String,
    /// 纯文本正文
    pub body: Option<String>,
    /// HTML 正文
    pub html_body: Option<String>,
    /// 附件列表
    pub attachments: Vec<Attachment>,
    /// 自定义头部字段
    pub headers: Vec<(String, String)>,
    /// 消息 ID
    pub message_id: Option<String>,
    /// 回复地址
    pub reply_to: Option<String>,
}

impl EmailMessage {
    /// 创建新的邮件消息
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用构建器创建邮件
    pub fn builder() -> EmailBuilder {
        EmailBuilder::new()
    }

    /// 生成完整的邮件内容（字节形式）
    ///
    /// 返回符合 RFC 5322 标准的邮件字节数据
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();

        self.write_headers(&mut output);

        let boundary = self.generate_boundary();

        if !self.attachments.is_empty() {
            self.write_multipart_mixed(&mut output, &boundary);
        }
        else if self.html_body.is_some() && self.body.is_some() {
            self.write_multipart_alternative(&mut output, &boundary);
        }
        else if let Some(ref html) = self.html_body {
            self.write_html_only(&mut output, html);
        }
        else if let Some(ref text) = self.body {
            self.write_text_only(&mut output, text);
        }
        else {
            output.extend_from_slice(b"\r\n");
        }

        output
    }

    /// 写入邮件头部
    fn write_headers(&self, output: &mut Vec<u8>) {
        if let Some(ref id) = self.message_id {
            output.extend_from_slice(format!("Message-ID: <{}>\r\n", id).as_bytes());
        }

        output.extend_from_slice(format!("From: {}\r\n", self.encode_address(&self.from)).as_bytes());

        if !self.to.is_empty() {
            let to_encoded: Vec<String> = self.to.iter().map(|a| self.encode_address(a)).collect();
            output.extend_from_slice(format!("To: {}\r\n", to_encoded.join(", ")).as_bytes());
        }

        if !self.cc.is_empty() {
            let cc_encoded: Vec<String> = self.cc.iter().map(|a| self.encode_address(a)).collect();
            output.extend_from_slice(format!("Cc: {}\r\n", cc_encoded.join(", ")).as_bytes());
        }

        if let Some(ref reply_to) = self.reply_to {
            output.extend_from_slice(format!("Reply-To: {}\r\n", self.encode_address(reply_to)).as_bytes());
        }

        output.extend_from_slice(format!("Subject: {}\r\n", encode_subject(&self.subject)).as_bytes());

        output.extend_from_slice(b"Date: ");
        output.extend_from_slice(generate_date().as_bytes());
        output.extend_from_slice(b"\r\n");

        output.extend_from_slice(b"MIME-Version: 1.0\r\n");

        for (name, value) in &self.headers {
            output.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
        }
    }

    /// 编码邮件地址（处理非 ASCII 字符）
    fn encode_address(&self, addr: &str) -> String {
        if addr.is_ascii() {
            addr.to_string()
        }
        else {
            if let Some(at_pos) = addr.rfind('@') {
                let name_part = &addr[..at_pos];
                let domain_part = &addr[at_pos..];
                if name_part.is_ascii() { addr.to_string() } else { format!("{}{}", encode_subject(name_part), domain_part) }
            }
            else {
                encode_subject(addr)
            }
        }
    }

    /// 生成 boundary 字符串
    fn generate_boundary(&self) -> String {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
        format!("----=_Part_{}_{}", timestamp, rand_suffix())
    }

    /// 写入纯文本邮件
    fn write_text_only(&self, output: &mut Vec<u8>, text: &str) {
        if text.is_ascii() {
            output.extend_from_slice(b"Content-Type: text/plain; charset=utf-8\r\n");
            output.extend_from_slice(b"Content-Transfer-Encoding: 7bit\r\n");
            output.extend_from_slice(b"\r\n");
            output.extend_from_slice(text.as_bytes());
        }
        else {
            output.extend_from_slice(b"Content-Type: text/plain; charset=utf-8\r\n");
            output.extend_from_slice(b"Content-Transfer-Encoding: base64\r\n");
            output.extend_from_slice(b"\r\n");
            let encoded = STANDARD.encode(text.as_bytes());
            for line in encoded.as_bytes().chunks(76) {
                output.extend_from_slice(line);
                output.extend_from_slice(b"\r\n");
            }
        }
    }

    /// 写入 HTML 邮件
    fn write_html_only(&self, output: &mut Vec<u8>, html: &str) {
        if html.is_ascii() {
            output.extend_from_slice(b"Content-Type: text/html; charset=utf-8\r\n");
            output.extend_from_slice(b"Content-Transfer-Encoding: 7bit\r\n");
            output.extend_from_slice(b"\r\n");
            output.extend_from_slice(html.as_bytes());
        }
        else {
            output.extend_from_slice(b"Content-Type: text/html; charset=utf-8\r\n");
            output.extend_from_slice(b"Content-Transfer-Encoding: base64\r\n");
            output.extend_from_slice(b"\r\n");
            let encoded = STANDARD.encode(html.as_bytes());
            for line in encoded.as_bytes().chunks(76) {
                output.extend_from_slice(line);
                output.extend_from_slice(b"\r\n");
            }
        }
    }

    /// 写入 multipart/alternative 消息（纯文本 + HTML）
    fn write_multipart_alternative(&self, output: &mut Vec<u8>, boundary: &str) {
        output.extend_from_slice(format!("Content-Type: multipart/alternative; boundary=\"{}\"\r\n", boundary).as_bytes());
        output.extend_from_slice(b"\r\n");

        output.extend_from_slice(b"This is a multi-part message in MIME format.\r\n\r\n");

        if let Some(ref text) = self.body {
            output.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            self.write_text_only(output, text);
            output.extend_from_slice(b"\r\n");
        }

        if let Some(ref html) = self.html_body {
            output.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            self.write_html_only(output, html);
            output.extend_from_slice(b"\r\n");
        }

        output.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
    }

    /// 写入 multipart/mixed 消息（包含附件）
    fn write_multipart_mixed(&self, output: &mut Vec<u8>, boundary: &str) {
        output.extend_from_slice(format!("Content-Type: multipart/mixed; boundary=\"{}\"\r\n", boundary).as_bytes());
        output.extend_from_slice(b"\r\n");

        output.extend_from_slice(b"This is a multi-part message in MIME format.\r\n\r\n");

        let has_content = self.body.is_some() || self.html_body.is_some();

        if has_content {
            output.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());

            if self.html_body.is_some() && self.body.is_some() {
                let alt_boundary = format!("{}_alt", boundary);
                output.extend_from_slice(
                    format!("Content-Type: multipart/alternative; boundary=\"{}\"\r\n", alt_boundary).as_bytes(),
                );
                output.extend_from_slice(b"\r\n");

                if let Some(ref text) = self.body {
                    output.extend_from_slice(format!("--{}\r\n", alt_boundary).as_bytes());
                    self.write_text_only(output, text);
                    output.extend_from_slice(b"\r\n");
                }

                if let Some(ref html) = self.html_body {
                    output.extend_from_slice(format!("--{}\r\n", alt_boundary).as_bytes());
                    self.write_html_only(output, html);
                    output.extend_from_slice(b"\r\n");
                }

                output.extend_from_slice(format!("--{}--\r\n\r\n", alt_boundary).as_bytes());
            }
            else if let Some(ref html) = self.html_body {
                self.write_html_only(output, html);
                output.extend_from_slice(b"\r\n");
            }
            else if let Some(ref text) = self.body {
                self.write_text_only(output, text);
                output.extend_from_slice(b"\r\n");
            }
        }

        for attachment in &self.attachments {
            output.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            self.write_attachment(output, attachment);
        }

        output.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
    }

    /// 写入附件
    fn write_attachment(&self, output: &mut Vec<u8>, attachment: &Attachment) {
        let filename_encoded = encode_subject(&attachment.filename);
        output.extend_from_slice(
            format!("Content-Type: {}; name=\"{}\"\r\n", attachment.content_type, filename_encoded).as_bytes(),
        );
        output.extend_from_slice(b"Content-Transfer-Encoding: base64\r\n");
        output.extend_from_slice(format!("Content-Disposition: attachment; filename=\"{}\"\r\n", filename_encoded).as_bytes());
        output.extend_from_slice(b"\r\n");

        let encoded = STANDARD.encode(&attachment.data);
        for line in encoded.as_bytes().chunks(76) {
            output.extend_from_slice(line);
            output.extend_from_slice(b"\r\n");
        }
        output.extend_from_slice(b"\r\n");
    }
}

impl fmt::Display for EmailMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.to_bytes()))
    }
}

/// 邮件构建器
///
/// 支持链式调用构建邮件消息。
#[derive(Debug, Clone, Default)]
pub struct EmailBuilder {
    message: EmailMessage,
}

impl EmailBuilder {
    /// 创建新的邮件构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置发件人
    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.message.from = from.into();
        self
    }

    /// 添加收件人
    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.message.to.push(to.into());
        self
    }

    /// 添加多个收件人
    pub fn to_multiple(mut self, addresses: Vec<String>) -> Self {
        self.message.to.extend(addresses);
        self
    }

    /// 添加抄送
    pub fn cc(mut self, cc: impl Into<String>) -> Self {
        self.message.cc.push(cc.into());
        self
    }

    /// 添加密送
    pub fn bcc(mut self, bcc: impl Into<String>) -> Self {
        self.message.bcc.push(bcc.into());
        self
    }

    /// 设置邮件主题
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.message.subject = subject.into();
        self
    }

    /// 设置纯文本正文
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.message.body = Some(body.into());
        self
    }

    /// 设置 HTML 正文
    pub fn html_body(mut self, html: impl Into<String>) -> Self {
        self.message.html_body = Some(html.into());
        self
    }

    /// 添加附件
    pub fn attachment(mut self, attachment: Attachment) -> Self {
        self.message.attachments.push(attachment);
        self
    }

    /// 添加多个附件
    pub fn attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.message.attachments.extend(attachments);
        self
    }

    /// 设置消息 ID
    pub fn message_id(mut self, id: impl Into<String>) -> Self {
        self.message.message_id = Some(id.into());
        self
    }

    /// 设置回复地址
    pub fn reply_to(mut self, reply_to: impl Into<String>) -> Self {
        self.message.reply_to = Some(reply_to.into());
        self
    }

    /// 添加自定义头部
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.message.headers.push((name.into(), value.into()));
        self
    }

    /// 构建邮件消息
    pub fn build(self) -> EmailMessage {
        self.message
    }
}

/// 使用 RFC 2047 编码主题（支持 UTF-8）
///
/// 格式: =?UTF-8?B?base64_encoded_text?=
pub fn encode_subject(subject: &str) -> String {
    if subject.is_ascii() {
        subject.to_string()
    }
    else {
        let encoded = STANDARD.encode(subject.as_bytes());
        format!("=?UTF-8?B?{}?=", encoded)
    }
}

/// 生成当前时间的 RFC 5322 格式日期字符串
pub fn generate_date() -> String {
    chrono_now()
}

/// 获取当前时间的格式化字符串
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    let days_since_epoch = secs / 86400;
    let secs_of_day = secs % 86400;
    let hours = secs_of_day / 3600;
    let minutes = (secs_of_day % 3600) / 60;
    let seconds = secs_of_day % 60;

    let (year, month, day, weekday) = days_to_date(days_since_epoch as i64);

    let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

    format!(
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} +0800",
        weekdays[weekday as usize],
        day,
        months[(month - 1) as usize],
        year,
        hours,
        minutes,
        seconds
    )
}

/// 将 Unix 天数转换为日期
fn days_to_date(days: i64) -> (i32, i32, i32, i32) {
    let mut year = 1970;
    let mut days_left = days;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days_left < days_in_year {
            break;
        }
        days_left -= days_in_year;
        year += 1;
    }

    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    }
    else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &days_in_month in &days_in_months {
        if days_left < days_in_month as i64 {
            break;
        }
        days_left -= days_in_month as i64;
        month += 1;
    }

    let day = days_left + 1;

    let days_since_epoch = days;
    let weekday = ((days_since_epoch + 3) % 7) as i32;
    let weekday = if weekday < 0 { weekday + 7 } else { weekday };

    (year, month, day as i32, weekday)
}

/// 判断是否为闰年
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// 生成随机后缀
fn rand_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let nanos = duration.subsec_nanos();

    format!("{:08x}", nanos.wrapping_mul(2654435761))
}
