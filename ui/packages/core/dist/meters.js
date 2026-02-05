function linearToDb(linear, floor = -60) {
  if (linear <= 0) {
    return floor;
  }
  const db = 20 * Math.log10(linear);
  return Math.max(db, floor);
}
function dbToLinear(db) {
  return Math.pow(10, db / 20);
}
export {
  dbToLinear,
  linearToDb
};
//# sourceMappingURL=meters.js.map
