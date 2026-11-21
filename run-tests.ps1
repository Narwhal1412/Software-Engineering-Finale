# Chạy test backend + frontend trong Docker rồi dọn dẹp. Mã thoát khác 0 nếu có test lỗi.
$f = "compose.test.yaml"
$status = 0

docker compose -f $f run --rm backend-tests
if ($LASTEXITCODE -ne 0) { $status = 1 }
docker compose -f $f run --rm frontend-tests
if ($LASTEXITCODE -ne 0) { $status = 1 }
docker compose -f $f down

if ($status -eq 0) { Write-Host "TAT CA TEST DEU PASS" } else { Write-Host "CO TEST LOI" }
exit $status
