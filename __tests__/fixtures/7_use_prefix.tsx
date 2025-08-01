export function selectUser() {
  const hasPermission = userHasPermission();

  return hasPermission;
}
