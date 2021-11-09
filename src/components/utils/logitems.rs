use asyncgit::sync::{CommitId, CommitInfo};
use std::slice::Iter;
use time::{Duration, OffsetDateTime};

use crate::components::utils::emojifi_string;

static SLICE_OFFSET_RELOAD_THRESHOLD: usize = 100;

type BoxStr = Box<str>;

pub struct LogEntry {
	//TODO: cache string representation
	pub time: OffsetDateTime,
	//TODO: use tinyvec here
	pub author: BoxStr,
	pub msg: BoxStr,
	//TODO: use tinyvec here
	pub hash_short: BoxStr,
	pub id: CommitId,
}

impl From<CommitInfo> for LogEntry {
	fn from(c: CommitInfo) -> Self {
		let time = OffsetDateTime::from_unix_timestamp(c.time)
			.expect("TODO");

		// Replace markdown emojis with Unicode equivalent
		let author = c.author;
		let mut msg = c.message;
		emojifi_string(&mut msg);

		Self {
			author: author.into(),
			msg: msg.into(),
			time,
			hash_short: c.id.get_short_string().into(),
			id: c.id,
		}
	}
}

static FORMAT_DATE: &[::time::format_description::FormatItem<'_>] =
	time::macros::format_description!("[year]-[month]-[day]");

static FORMAT_TIME: &[::time::format_description::FormatItem<'_>] =
	time::macros::format_description!("[hour]:[minute]:[second]  ");

impl LogEntry {
	pub fn time_to_string(
		&self,
		now: time::OffsetDateTime,
	) -> String {
		let delta = now - self.time;
		if delta < Duration::minutes(30) {
			let delta_str = if delta < Duration::minutes(1) {
				"<1m ago".to_string()
			} else {
				format!("{:0>2}m ago", delta.whole_minutes())
			};
			format!("{: <10}", delta_str)
		} else if self.time.date() == now.date() {
			self.time.format(&FORMAT_TIME).expect("TODO")
		} else {
			self.time.format(&FORMAT_DATE).expect("TODO")
		}
	}
}

///
#[derive(Default)]
pub struct ItemBatch {
	index_offset: usize,
	items: Vec<LogEntry>,
}

impl ItemBatch {
	fn last_idx(&self) -> usize {
		self.index_offset + self.items.len()
	}

	///
	pub const fn index_offset(&self) -> usize {
		self.index_offset
	}

	/// shortcut to get an `Iter` of our internal items
	pub fn iter(&self) -> Iter<'_, LogEntry> {
		self.items.iter()
	}

	/// clear curent list of items
	pub fn clear(&mut self) {
		self.items.clear();
	}

	/// insert new batch of items
	pub fn set_items(
		&mut self,
		start_index: usize,
		commits: Vec<CommitInfo>,
	) {
		self.items.clear();
		self.items.extend(commits.into_iter().map(LogEntry::from));
		self.index_offset = start_index;
	}

	/// returns `true` if we should fetch updated list of items
	pub fn needs_data(&self, idx: usize, idx_max: usize) -> bool {
		let want_min =
			idx.saturating_sub(SLICE_OFFSET_RELOAD_THRESHOLD);
		let want_max = idx
			.saturating_add(SLICE_OFFSET_RELOAD_THRESHOLD)
			.min(idx_max);

		let needs_data_top = want_min < self.index_offset;
		let needs_data_bottom = want_max >= self.last_idx();
		needs_data_bottom || needs_data_top
	}
}

#[cfg(test)]
mod tests {
	use time::macros::datetime;

	use super::*;

	fn test_conversion(s: &str) -> String {
		let mut s = s.to_string();
		emojifi_string(&mut s);
		s
	}

	#[test]
	fn test_emojifi_string_conversion_cases() {
		assert_eq!(
			&test_conversion("It's :hammer: time!"),
			"It's 🔨 time!"
		);
		assert_eq!(
			&test_conversion(":red_circle::orange_circle::yellow_circle::green_circle::large_blue_circle::purple_circle:"),
			"🔴🟠🟡🟢🔵🟣"
		);
		assert_eq!(
			&test_conversion("It's raining :cat:s and :dog:s"),
			"It's raining 🐱s and 🐶s"
		);
		assert_eq!(&test_conversion(":crab: rules!"), "🦀 rules!");
	}

	#[test]
	fn test_emojifi_string_no_conversion_cases() {
		assert_eq!(&test_conversion("123"), "123");
		assert_eq!(
			&test_conversion("This :should_not_convert:"),
			"This :should_not_convert:"
		);
		assert_eq!(&test_conversion(":gopher:"), ":gopher:");
	}

	#[test]
	fn test_time() {
		let entry = LogEntry::from(CommitInfo {
			author: String::new(),
			message: String::new(),
			id: CommitId::default(),
			time: 0,
		});

		assert_eq!(
			&entry.time_to_string(datetime!(2000-01-01 0:00 UTC)),
			"1970-01-01"
		);
		assert_eq!(
			&entry.time_to_string(datetime!(1970-01-01 1:00 UTC)),
			"00:00:00  "
		);
		assert_eq!(
			&entry.time_to_string(datetime!(1970-01-01 0:02 UTC)),
			"02m ago   "
		);
		assert_eq!(
			&entry.time_to_string(datetime!(1970-01-01 0:00:01 UTC)),
			"<1m ago   "
		);
	}
}
