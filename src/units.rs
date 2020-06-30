use crate::{geometry::Point2, ids::UnitTypeId, unit::Unit};
use indexmap::{
	map::{IntoIter, Iter, IterMut, Keys, Values, ValuesMut},
	IndexMap,
};
use itertools::Itertools;
use rustc_hash::FxHasher;
use std::{
	hash::BuildHasherDefault,
	iter::{FromIterator, Sum},
	ops::{Index, IndexMut},
};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Default, Clone)]
pub struct AllUnits {
	pub all: Units,
	pub my: PlayerUnits,
	pub enemy: PlayerUnits,
	pub mineral_fields: Units,
	pub vespene_geysers: Units,
	pub resources: Units,
	pub destructables: Units,
	pub watchtowers: Units,
	pub inhibitor_zones: Units,
}
impl AllUnits {
	pub(crate) fn clear(&mut self) {
		self.all.clear();
		self.my.clear();
		self.enemy.clear();
		self.mineral_fields.clear();
		self.vespene_geysers.clear();
		self.resources.clear();
		self.destructables.clear();
		self.watchtowers.clear();
		self.inhibitor_zones.clear();
	}
}
#[derive(Default, Clone)]
pub struct PlayerUnits {
	pub all: Units,
	pub units: Units,
	pub structures: Units,
	pub townhalls: Units,
	pub workers: Units,
	pub gas_buildings: Units,
	pub larvas: Units,
	pub placeholders: Units,
}
impl PlayerUnits {
	pub(crate) fn clear(&mut self) {
		self.all.clear();
		self.units.clear();
		self.structures.clear();
		self.townhalls.clear();
		self.workers.clear();
		self.gas_buildings.clear();
		self.larvas.clear();
		self.placeholders.clear();
	}
}

#[derive(Default, Clone)]
pub struct Units(FxIndexMap<u64, Unit>);
impl Units {
	// HashMap methods
	#[inline]
	pub fn new() -> Self {
		Units(FxIndexMap::default())
	}

	#[inline]
	pub fn first(&self) -> Option<&Unit> {
		self.0.values().next()
	}

	#[inline]
	pub fn push(&mut self, u: Unit) -> Option<Unit> {
		self.0.insert(u.tag, u)
	}

	#[inline]
	pub fn pop(&mut self) -> Option<Unit> {
		self.0.pop().map(|i| i.1)
	}

	#[inline]
	pub fn remove(&mut self, u: u64) -> Option<Unit> {
		self.0.remove(&u)
	}

	#[inline]
	pub fn iter(&self) -> Values<u64, Unit> {
		self.0.values()
	}

	#[inline]
	pub fn iter_mut(&mut self) -> ValuesMut<u64, Unit> {
		self.0.values_mut()
	}

	#[inline]
	pub fn pairs(&self) -> Iter<u64, Unit> {
		self.0.iter()
	}

	#[inline]
	pub fn pairs_mut(&mut self) -> IterMut<u64, Unit> {
		self.0.iter_mut()
	}

	#[inline]
	pub fn tags(&self) -> Keys<u64, Unit> {
		self.0.keys()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	#[inline]
	pub fn clear(&mut self) {
		self.0.clear()
	}

	// Units methods
	pub fn contains_tag(&self, tag: u64) -> bool {
		self.0.contains_key(&tag)
	}
	pub fn find_tag(&self, tag: u64) -> Option<&Unit> {
		self.0.get(&tag)
	}
	pub fn find_tags<'a, T: IntoIterator<Item = &'a u64>>(&self, tags: T) -> Self {
		tags.into_iter()
			.filter_map(|tag| self.0.get(tag).cloned())
			.collect()
	}
	pub fn of_type(&self, u_type: UnitTypeId) -> Self {
		self.filter(|u| u.type_id == u_type)
	}
	pub fn of_types<T: Container<UnitTypeId>>(&self, types: T) -> Self {
		self.filter(|u| types.contains(&u.type_id))
	}
	pub fn center(&self) -> Option<Point2> {
		if self.is_empty() {
			None
		} else {
			Some(self.sum(|u| u.position) / self.len() as f32)
		}
	}
	// Get closest | furthest
	pub fn closest<P: Into<Point2> + Copy>(&self, target: P) -> Option<&Unit> {
		self.partial_min(|u| u.distance_squared(target))
	}
	pub fn furthest<P: Into<Point2> + Copy>(&self, target: P) -> Option<&Unit> {
		self.partial_max(|u| u.distance_squared(target))
	}
	// Get closest | furthest distance
	pub fn closest_distance<P: Into<Point2> + Copy>(&self, target: P) -> Option<f32> {
		self.partial_min_value(|u| u.distance_squared(target))
			.map(|dist| dist.sqrt())
	}
	pub fn furthest_distance<P: Into<Point2> + Copy>(&self, target: P) -> Option<f32> {
		self.partial_max_value(|u| u.distance_squared(target))
			.map(|dist| dist.sqrt())
	}
	// Squared
	pub fn closest_distance_squared<P: Into<Point2> + Copy>(&self, target: P) -> Option<f32> {
		self.partial_min_value(|u| u.distance_squared(target))
	}
	pub fn furthest_distance_squared<P: Into<Point2> + Copy>(&self, target: P) -> Option<f32> {
		self.partial_max_value(|u| u.distance_squared(target))
	}
	// Filter closer | further than distance
	pub fn closer<P: Into<Point2> + Copy>(&self, distance: f32, target: P) -> Units {
		self.filter(|u| u.is_closer(distance, target))
	}
	pub fn further<P: Into<Point2> + Copy>(&self, distance: f32, target: P) -> Units {
		self.filter(|u| u.is_further(distance, target))
	}

	pub fn filter<F>(&self, f: F) -> Self
	where
		F: Fn(&&Unit) -> bool,
	{
		Self(self.iter().filter(f).map(|u| (u.tag, u.clone())).collect())
	}
	pub fn ground(&self) -> Self {
		self.filter(|u| !u.is_flying)
	}
	pub fn flying(&self) -> Self {
		self.filter(|u| u.is_flying)
	}
	pub fn ready(&self) -> Self {
		self.filter(|u| u.is_ready())
	}
	pub fn not_ready(&self) -> Self {
		self.filter(|u| !u.is_ready())
	}
	pub fn idle(&self) -> Self {
		self.filter(|u| u.is_idle())
	}
	pub fn almost_idle(&self) -> Self {
		self.filter(|u| u.is_almost_idle())
	}
	pub fn unused(&self) -> Self {
		self.filter(|u| u.is_unused())
	}
	pub fn almost_unused(&self) -> Self {
		self.filter(|u| u.is_almost_unused())
	}
	pub fn in_range_of(&self, unit: &Unit, gap: f32) -> Self {
		self.filter(|u| unit.in_range(u, gap))
	}
	pub fn in_range(&self, unit: &Unit, gap: f32) -> Self {
		self.filter(|u| u.in_range(unit, gap))
	}
	pub fn visible(&self) -> Self {
		self.filter(|u| u.is_visible())
	}
	pub fn sum<T, F>(&self, f: F) -> T
	where
		T: Sum,
		F: Fn(&Unit) -> T,
	{
		self.iter().map(f).sum::<T>()
	}
	pub fn min<T, F>(&self, f: F) -> Option<&Unit>
	where
		T: Ord,
		F: Fn(&&Unit) -> T,
	{
		self.iter().min_by_key(f)
	}
	pub fn partial_min<T, F>(&self, f: F) -> Option<&Unit>
	where
		T: PartialOrd,
		F: Fn(&Unit) -> T,
	{
		self.iter().min_by(|u1, u2| f(u1).partial_cmp(&f(u2)).unwrap())
	}
	pub fn min_value<T, F>(&self, f: F) -> Option<T>
	where
		T: Ord,
		F: Fn(&Unit) -> T,
	{
		self.iter().map(f).min()
	}
	pub fn partial_min_value<T, F>(&self, f: F) -> Option<T>
	where
		T: PartialOrd,
		F: Fn(&Unit) -> T,
	{
		self.iter().map(f).min_by(|a, b| a.partial_cmp(&b).unwrap())
	}
	pub fn max<T, F>(&self, f: F) -> Option<&Unit>
	where
		T: Ord,
		F: Fn(&&Unit) -> T,
	{
		self.iter().max_by_key(f)
	}
	pub fn partial_max<T, F>(&self, f: F) -> Option<&Unit>
	where
		T: PartialOrd,
		F: Fn(&Unit) -> T,
	{
		self.iter().max_by(|u1, u2| f(u1).partial_cmp(&f(u2)).unwrap())
	}
	pub fn max_value<T, F>(&self, f: F) -> Option<T>
	where
		T: Ord,
		F: Fn(&Unit) -> T,
	{
		self.iter().map(f).max()
	}
	pub fn partial_max_value<T, F>(&self, f: F) -> Option<T>
	where
		T: PartialOrd,
		F: Fn(&Unit) -> T,
	{
		self.iter().map(f).max_by(|a, b| a.partial_cmp(&b).unwrap())
	}
	pub fn sort<T, F>(&self, f: F) -> Self
	where
		T: Ord,
		F: Fn(&&Unit) -> T,
	{
		self.iter().sorted_by_key(f).cloned().collect()
	}
	pub fn partial_sort<T, F>(&self, f: F) -> Self
	where
		T: PartialOrd,
		F: Fn(&Unit) -> T,
	{
		self.iter()
			.sorted_by(|u1, u2| f(u1).partial_cmp(&f(u2)).unwrap())
			.cloned()
			.collect()
	}
	pub fn sort_unstable<T, F>(&self, f: F) -> Self
	where
		T: Ord,
		F: Fn(&&Unit) -> T,
	{
		let mut v = Vec::from_iter(self.iter());
		v.sort_unstable_by_key(f);
		v.into_iter().cloned().collect()
	}
	pub fn partial_sort_unstable<T, F>(&self, f: F) -> Self
	where
		T: PartialOrd,
		F: Fn(&Unit) -> T,
	{
		let mut v = Vec::from_iter(self.iter());
		v.sort_unstable_by(|u1, u2| f(u1).partial_cmp(&f(u2)).unwrap());
		v.into_iter().cloned().collect()
	}
}
impl FromIterator<Unit> for Units {
	#[inline]
	fn from_iter<I: IntoIterator<Item = Unit>>(iter: I) -> Self {
		Self(iter.into_iter().map(|u| (u.tag, u)).collect())
	}
}
impl IntoIterator for Units {
	type Item = (u64, Unit);
	type IntoIter = IntoIter<u64, Unit>;

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
impl Index<u64> for Units {
	type Output = Unit;

	#[inline]
	fn index(&self, tag: u64) -> &Self::Output {
		&self.0[&tag]
	}
}
impl IndexMut<u64> for Units {
	#[inline]
	fn index_mut(&mut self, tag: u64) -> &mut Self::Output {
		self.0.get_mut(&tag).unwrap()
	}
}
impl Extend<Unit> for Units {
	#[inline]
	fn extend<T: IntoIterator<Item = Unit>>(&mut self, iter: T) {
		self.0.extend(iter.into_iter().map(|u| (u.tag, u)));
	}
}

#[cfg(feature = "rayon")]
use indexmap::map::rayon::{IntoParIter, ParIter, ParIterMut, ParKeys, ParValues, ParValuesMut};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[cfg(feature = "rayon")]
impl Units {
	#[inline]
	pub fn par_iter(&self) -> ParValues<u64, Unit> {
		self.0.par_values()
	}

	#[inline]
	pub fn par_iter_mut(&mut self) -> ParValuesMut<u64, Unit> {
		self.0.par_values_mut()
	}

	#[inline]
	pub fn par_pairs(&self) -> ParIter<u64, Unit> {
		self.0.par_iter()
	}

	#[inline]
	pub fn par_pairs_mut(&mut self) -> ParIterMut<u64, Unit> {
		self.0.par_iter_mut()
	}

	#[inline]
	pub fn par_tags(&self) -> ParKeys<u64, Unit> {
		self.0.par_keys()
	}
}

#[cfg(feature = "rayon")]
impl IntoParallelIterator for Units {
	type Item = (u64, Unit);
	type Iter = IntoParIter<u64, Unit>;

	#[inline]
	fn into_par_iter(self) -> Self::Iter {
		self.0.into_par_iter()
	}
}

#[cfg(feature = "rayon")]
impl ParallelExtend<Unit> for Units {
	#[inline]
	fn par_extend<T: IntoParallelIterator<Item = Unit>>(&mut self, par_iter: T) {
		self.0.par_extend(par_iter.into_par_iter().map(|u| (u.tag, u)));
	}
}

#[cfg(feature = "rayon")]
impl FromParallelIterator<Unit> for Units {
	#[inline]
	fn from_par_iter<I: IntoParallelIterator<Item = Unit>>(par_iter: I) -> Self {
		Self(par_iter.into_par_iter().map(|u| (u.tag, u)).collect())
	}
}

pub trait Container<T> {
	fn contains(&self, item: &T) -> bool;
}

use std::{
	collections::{BTreeMap, BTreeSet, HashMap, HashSet},
	hash::{BuildHasher, Hash},
};

impl<T: PartialEq> Container<T> for [T] {
	fn contains(&self, other: &T) -> bool {
		self.iter().any(|item| item == other)
	}
}
impl<T: PartialEq> Container<T> for Vec<T> {
	fn contains(&self, other: &T) -> bool {
		self.iter().any(|item| item == other)
	}
}
impl<T: Eq + Hash, S: BuildHasher> Container<T> for HashSet<T, S> {
	fn contains(&self, item: &T) -> bool {
		self.contains(item)
	}
}
impl<T: Eq + Hash, V, S: BuildHasher> Container<T> for HashMap<T, V, S> {
	fn contains(&self, item: &T) -> bool {
		self.contains_key(item)
	}
}
impl<T: Ord> Container<T> for BTreeSet<T> {
	fn contains(&self, item: &T) -> bool {
		self.contains(item)
	}
}
impl<T: Ord, V> Container<T> for BTreeMap<T, V> {
	fn contains(&self, item: &T) -> bool {
		self.contains_key(item)
	}
}
