# 📖 mkrp 사용 가이드 (How to use mkrp)

`mkrp`는 RG VITA PRO(ROCKNIX), RG40XXH(Knulli), RG DS 등 Linux ARM64 핸드헬드 기기의 **PortMaster** 환경에서 **Ren'Py 비주얼 노벨 게임**을 쾌적하고 안정적으로 플레이할 수 있도록 돕는 실기기 세팅 가이드입니다.

---

## 🎮 빠른 시작 (PortMaster 표준 Ren'Py 환경)

### 1. 기본 디렉토리 구조
기기의 SD 카드 내 `ports` 디렉토리에 다음과 같이 게임 폴더를 구성합니다:

```text
roms/ (또는 storage/)
  ports/
    GameName.sh                <-- PortMaster 실행 런처 스크립트 (또는 mkrp.sh)
    GameName/                  <-- 게임 디렉토리
      game/                    <-- ⭐️ 게임 본편 에셋 및 스크립트 폴더
        *.rpa                  <-- 게임 리소스 아카이브 파일들
        patch_assets.rpa       <-- 변환된 이미지 패치 아카이브 (선택)
        zz_00_handheld_core.rpy  <-- 핸드헬드 패드/메모리 최적화 스크립트
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

1. **Python 2 ➔ 3 스탯 비교 및 screen 호환성**:
   - Ren'Py 7 구버전 게임의 스탯/호감도 비교 구문(`__cmp__`, `< None`) 충돌 및 `renpy.has_screen` 속성 누락 에러는 `zz_00_handheld_core.rpy`가 자동으로 감지하여 완벽히 방어합니다.
2. **저사양 RAM 기기 메모리 관리**:
   - `zz_00_handheld_core.rpy`가 부팅 시 `/proc/meminfo`를 확인하여 1GB 기기(160MB 캐시, 보수적 프리딕트)부터 4GB 기기(512MB 캐시)까지 자동으로 스케일링하여 메모리 부족(OOM) 강제 종료를 방지합니다.
3. **게임별 특화 패치 오버레이 (`patches/<game_id>/`)**:
   - 특정 게임에만 필요한 고유 최적화 스크립트가 있다면 프로젝트 루트의 `patches/<게임ID>/` 디렉토리에 패치 파일을 두고 `pnpm run build --patch <게임ID>`로 빌드하면 배포 패키지에 자동으로 합쳐집니다 (`patches/` 디렉토리는 `.gitignore`로 제외되어 Git에 커밋되지 않습니다).
4. **PC 1080p 고해상도 에셋 최적화**:
   - 저사양 ARM 기기에서 대용량 이미지로 인한 로딩 지연을 해결하려면 PC에서 `pnpm run optimize --input <에셋폴더>`를 실행하여 720p/480p로 경량화하여 적용할 수 있습니다.
