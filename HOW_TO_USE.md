# 📖 mkrp 사용 가이드 (How to use mkrp)

`mkrp`는 RG VITA PRO(ROCKNIX), RG40XXH(Knulli), RG DS 등 Linux ARM64 핸드헬드 기기의 **PortMaster** 환경에서 **Ren'Py 비주얼 노벨 게임**을 쾌적하고 안정적으로 플레이할 수 있도록 돕는 실기기 세팅 가이드입니다.

---

## 🎮 빠른 시작 (PortMaster 표준 Ren'Py 환경)

### 1. 기본 디렉토리 구조
기기의 SD 카드 내 `ports` 디렉토리에 다음과 같이 게임 폴더를 구성합니다:

```text
roms/ (또는 storage/)
  ports/
    VIRTUES.sh                 <-- PortMaster 실행 런처 스크립트
    VIRTUES/                   <-- 게임 디렉토리
      renpy/                   <-- PortMaster 공식 Ren'Py 런타임 (SquashFS 마운트)
      game/                    <-- ⭐️ 게임 본편 에셋 및 스크립트 폴더
        *.rpa                  <-- 게임 리소스 아카이브 파일들
        patch_virtues_assets.rpa <-- 변환된 이미지 패치 아카이브 (선택)
        zz_avif_loader.rpy     <-- Ren'Py 8 하위 호환 및 가상 이미지 로더
        zz_handheld_patch.rpy  <-- 핸드헬드 패드/메모리 최적화 스크립트
      log.txt                  <-- 실행 로그 및 에러 출력 파일
```

---

## 🕹️ 기본 조작법 (핸드헬드 표준 배열)

| 버튼 | 기능 | 설명 |
| :--- | :--- | :--- |
| **A** | 확인 / 대사 진행 | 좌클릭 / Enter |
| **B** | 취소 / 백로그 롤백 | 우클릭 / PageUp |
| **X** | UI 숨기기 | 일러스트 전체화면 감상 |
| **Y** | 게임 메뉴 / 세이브 | ESC / 메뉴 열기 |
| **L1** | 대사 건너뛰기 (Skip) | 빠른 스킵 모드 |
| **R1** | 히스토리 (History) | 지나간 대사 로그 확인 |
| **L2** | 자동 진행 (Auto) | 자동 넘김 토글 |
| **R2** | 퀵 세이브 (Quick Save) | 빠른 저장 |
| **왼쪽 아날로그 스틱** | 가상 마우스 커서 이동 | 정밀 포인터 조작 |
| **D-Pad (십자키)** | 선택지 및 버튼 이동 | 메뉴 포커스 이동 |

---

## 💡 호환성 및 최적화 팁

1. **Python 2 ➔ 3 스탯 비교 호환성**:
   - Ren'Py 7 구버전 게임의 스탯/호감도 비교 구문(`__cmp__`, `< None`) 충돌을 방지하기 위해 `zz_avif_loader.rpy`를 `game/` 폴더에 포함하면 자동으로 방어됩니다.
2. **저사양 RAM 기기 메모리 관리**:
   - `zz_handheld_patch.rpy`에 `config.image_cache_size_mb = 128`이 설정되어 있어 1080p 고해상도 CG 로딩 중 발생할 수 있는 메모리 부족(OOM) 강제 종료를 방지합니다.
3. **AVIF 등 비표준 에셋 대응**:
   - SDL2가 지원하지 않는 이미지 포맷은 `patch_virtues_assets.rpa`처럼 PNG/WebP로 변환 후 패치 아카이브로 묶어 제공하면 `zz_avif_loader.rpy`가 자동으로 리다이렉트합니다.
