#!/usr/bin/env sh
# Chạy test backend + frontend trong Docker rồi dọn dẹp. Mã thoát khác 0 nếu có test lỗi.
F=compose.test.yaml
status=0

docker compose -f "$F" run --rm backend-tests || status=1
docker compose -f "$F" run --rm frontend-tests || status=1
docker compose -f "$F" down

if [ "$status" -eq 0 ]; then echo "TẤT CẢ TEST ĐỀU PASS"; else echo "CÓ TEST LỖI"; fi
exit "$status"
