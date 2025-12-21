/*
 * da7280.h - DA7280 Haptic Hardware Abstraction Layer Header
 *
 * Copyright (C) 2017 Dialog Semiconductor Ltd. and its Affiliates, unpublished
 * work. This computer program includes Confidential, Proprietary Information
 * and is a Trade Secret of Dialog Semiconductor Ltd. and its Affiliates.
 * All use, disclosure, and/or reproduction is prohibited unless authorized
 * in writing. All Rights Reserved.
 */

#ifndef _DA7280_REG_DEFS_H
#define _DA7280_REG_DEFS_H

#define DA7280_CHIP_REV                       0x0000
#define DA7280_IRQ_EVENT1                     0x0003
#define DA7280_IRQ_EVENT_WARNING_DIAG         0x0004
#define DA7280_IRQ_EVENT_PAT_DIAG             0x0005
#define DA7280_IRQ_STATUS1                    0x0006
#define DA7280_IRQ_MASK1                      0x0007
#define DA7280_CIF_I2C1                       0x0008
#define DA7280_FRQ_LRA_PER_H                  0x000A
#define DA7280_FRQ_LRA_PER_L                  0x000B
#define DA7280_ACTUATOR1                      0x000C
#define DA7280_ACTUATOR2                      0x000D
#define DA7280_ACTUATOR3                      0x000E
#define DA7280_CALIB_V2I_H                    0x000F
#define DA7280_CALIB_V2I_L                    0x0010
#define DA7280_CALIB_IMP_H                    0x0011
#define DA7280_CALIB_IMP_L                    0x0012
#define DA7280_TOP_CFG1                       0x0013
#define DA7280_TOP_CFG2                       0x0014
#define DA7280_TOP_CFG3                       0x0015
#define DA7280_TOP_CFG4                       0x0016
#define DA7280_TOP_INT_CFG1                   0x0017
#define DA7280_TOP_INT_CFG6_H                 0x001C
#define DA7280_TOP_INT_CFG6_L                 0x001D
#define DA7280_TOP_INT_CFG7_H                 0x001E
#define DA7280_TOP_INT_CFG7_L                 0x001F
#define DA7280_TOP_INT_CFG8                   0x0020
#define DA7280_TOP_CTL1                       0x0022
#define DA7280_TOP_CTL2                       0x0023
#define DA7280_SEQ_CTL1                       0x0024
#define DA7280_SWG_C1                         0x0025
#define DA7280_SWG_C2                         0x0026
#define DA7280_SWG_C3                         0x0027
#define DA7280_SEQ_CTL2                       0x0028
#define DA7280_GPI_0_CTL                      0x0029
#define DA7280_GPI_1_CTL                      0x002A
#define DA7280_GPI_2_CTL                      0x002B
#define DA7280_MEM_CTL1                       0x002C
#define DA7280_MEM_CTL2                       0x002D
#define DA7280_ADC_DATA_H1                    0x002E
#define DA7280_ADC_DATA_L1                    0x002F
#define DA7280_POLARITY                       0x0043
#define DA7280_LRA_AVR_H                      0x0044
#define DA7280_LRA_AVR_L                      0x0045
#define DA7280_FRQ_LRA_PER_ACT_H              0x0046
#define DA7280_FRQ_LRA_PER_ACT_L              0x0047
#define DA7280_FRQ_PHASE_H                    0x0048
#define DA7280_FRQ_PHASE_L                    0x0049
#define DA7280_FRQ_CTL                        0x004C
#define DA7280_TRIM3                          0x005F
#define DA7280_TRIM4                          0x0060
#define DA7280_TRIM6                          0x0062
#define DA7280_IRQ_EVENT_ACTUATOR_FAULT       0x0081
#define DA7280_IRQ_STATUS2                    0x0082
#define DA7280_IRQ_MASK2                      0x0083
#define DA7280_SNP_MEM_0                      0x0084
#define DA7280_SNP_MEM_1                      0x0085
#define DA7280_SNP_MEM_2                      0x0086
#define DA7280_SNP_MEM_3                      0x0087
#define DA7280_SNP_MEM_4                      0x0088
#define DA7280_SNP_MEM_5                      0x0089
#define DA7280_SNP_MEM_6                      0x008A
#define DA7280_SNP_MEM_7                      0x008B
#define DA7280_SNP_MEM_8                      0x008C
#define DA7280_SNP_MEM_9                      0x008D
#define DA7280_SNP_MEM_10                     0x008E
#define DA7280_SNP_MEM_11                     0x008F
#define DA7280_SNP_MEM_12                     0x0090
#define DA7280_SNP_MEM_13                     0x0091
#define DA7280_SNP_MEM_14                     0x0092
#define DA7280_SNP_MEM_15                     0x0093
#define DA7280_SNP_MEM_16                     0x0094
#define DA7280_SNP_MEM_17                     0x0095
#define DA7280_SNP_MEM_18                     0x0096
#define DA7280_SNP_MEM_19                     0x0097
#define DA7280_SNP_MEM_20                     0x0098
#define DA7280_SNP_MEM_21                     0x0099
#define DA7280_SNP_MEM_22                     0x009A
#define DA7280_SNP_MEM_23                     0x009B
#define DA7280_SNP_MEM_24                     0x009C
#define DA7280_SNP_MEM_25                     0x009D
#define DA7280_SNP_MEM_26                     0x009E
#define DA7280_SNP_MEM_27                     0x009F
#define DA7280_SNP_MEM_28                     0x00A0
#define DA7280_SNP_MEM_29                     0x00A1
#define DA7280_SNP_MEM_30                     0x00A2
#define DA7280_SNP_MEM_31                     0x00A3
#define DA7280_SNP_MEM_32                     0x00A4
#define DA7280_SNP_MEM_33                     0x00A5
#define DA7280_SNP_MEM_34                     0x00A6
#define DA7280_SNP_MEM_35                     0x00A7
#define DA7280_SNP_MEM_36                     0x00A8
#define DA7280_SNP_MEM_37                     0x00A9
#define DA7280_SNP_MEM_38                     0x00AA
#define DA7280_SNP_MEM_39                     0x00AB
#define DA7280_SNP_MEM_40                     0x00AC
#define DA7280_SNP_MEM_41                     0x00AD
#define DA7280_SNP_MEM_42                     0x00AE
#define DA7280_SNP_MEM_43                     0x00AF
#define DA7280_SNP_MEM_44                     0x00B0
#define DA7280_SNP_MEM_45                     0x00B1
#define DA7280_SNP_MEM_46                     0x00B2
#define DA7280_SNP_MEM_47                     0x00B3
#define DA7280_SNP_MEM_48                     0x00B4
#define DA7280_SNP_MEM_49                     0x00B5
#define DA7280_SNP_MEM_50                     0x00B6
#define DA7280_SNP_MEM_51                     0x00B7
#define DA7280_SNP_MEM_52                     0x00B8
#define DA7280_SNP_MEM_53                     0x00B9
#define DA7280_SNP_MEM_54                     0x00BA
#define DA7280_SNP_MEM_55                     0x00BB
#define DA7280_SNP_MEM_56                     0x00BC
#define DA7280_SNP_MEM_57                     0x00BD
#define DA7280_SNP_MEM_58                     0x00BE
#define DA7280_SNP_MEM_59                     0x00BF
#define DA7280_SNP_MEM_60                     0x00C0
#define DA7280_SNP_MEM_61                     0x00C1
#define DA7280_SNP_MEM_62                     0x00C2
#define DA7280_SNP_MEM_63                     0x00C3
#define DA7280_SNP_MEM_64                     0x00C4
#define DA7280_SNP_MEM_65                     0x00C5
#define DA7280_SNP_MEM_66                     0x00C6
#define DA7280_SNP_MEM_67                     0x00C7
#define DA7280_SNP_MEM_68                     0x00C8
#define DA7280_SNP_MEM_69                     0x00C9
#define DA7280_SNP_MEM_70                     0x00CA
#define DA7280_SNP_MEM_71                     0x00CB
#define DA7280_SNP_MEM_72                     0x00CC
#define DA7280_SNP_MEM_73                     0x00CD
#define DA7280_SNP_MEM_74                     0x00CE
#define DA7280_SNP_MEM_75                     0x00CF
#define DA7280_SNP_MEM_76                     0x00D0
#define DA7280_SNP_MEM_77                     0x00D1
#define DA7280_SNP_MEM_78                     0x00D2
#define DA7280_SNP_MEM_79                     0x00D3
#define DA7280_SNP_MEM_80                     0x00D4
#define DA7280_SNP_MEM_81                     0x00D5
#define DA7280_SNP_MEM_82                     0x00D6
#define DA7280_SNP_MEM_83                     0x00D7
#define DA7280_SNP_MEM_84                     0x00D8
#define DA7280_SNP_MEM_85                     0x00D9
#define DA7280_SNP_MEM_86                     0x00DA
#define DA7280_SNP_MEM_87                     0x00DB
#define DA7280_SNP_MEM_88                     0x00DC
#define DA7280_SNP_MEM_89                     0x00DD
#define DA7280_SNP_MEM_90                     0x00DE
#define DA7280_SNP_MEM_91                     0x00DF
#define DA7280_SNP_MEM_92                     0x00E0
#define DA7280_SNP_MEM_93                     0x00E1
#define DA7280_SNP_MEM_94                     0x00E2
#define DA7280_SNP_MEM_95                     0x00E3
#define DA7280_SNP_MEM_96                     0x00E4
#define DA7280_SNP_MEM_97                     0x00E5
#define DA7280_SNP_MEM_98                     0x00E6
#define DA7280_SNP_MEM_99                     0x00E7

/* DA7280_CHIP_REV (Address 0x0000) */
#define DA7280_CHIP_REV_MAJOR_SHIFT                           0
#define DA7280_CHIP_REV_MAJOR_MASK                            (15 << 0)
#define DA7280_CHIP_REV_MINOR_SHIFT                           4
#define DA7280_CHIP_REV_MINOR_MASK                            (15 << 4)

/* DA7280_IRQ_EVENT1 (Address 0x0003) */
#define DA7280_E_SEQ_CONTINUE_SHIFT                         0
#define DA7280_E_SEQ_CONTINUE_MASK                          (1 << 0)
#define DA7280_E_UVLO_VBAT_OK_SHIFT                         1
#define DA7280_E_UVLO_VBAT_OK_MASK                          (1 << 1)
#define DA7280_E_PAT_DONE_SHIFT                             2
#define DA7280_E_PAT_DONE_MASK                              (1 << 2)
#define DA7280_E_OVERTEMP_CRIT_SHIFT                        3
#define DA7280_E_OVERTEMP_CRIT_MASK                         (1 << 3)
#define DA7280_E_PAT_FAULT_SHIFT                            4
#define DA7280_E_PAT_FAULT_MASK                             (1 << 4)
#define DA7280_E_WARNING_SHIFT                              5
#define DA7280_E_WARNING_MASK                               (1 << 5)
#define DA7280_E_ACTUATOR_FAULT_SHIFT                       6
#define DA7280_E_ACTUATOR_FAULT_MASK                        (1 << 6)
#define DA7280_E_OC_FAULT_SHIFT                             7
#define DA7280_E_OC_FAULT_MASK                              (1 << 7)

/* DA7280_IRQ_EVENT_WARNING_DIAG (Address 0x0004) */
#define DA7280_E_OVERTEMP_WARN_SHIFT            3
#define DA7280_E_OVERTEMP_WARN_MASK             (1 << 3)
#define DA7280_E_MEM_TYPE_SHIFT                 4
#define DA7280_E_MEM_TYPE_MASK                  (1 << 4)
#define DA7280_E_LIM_DRIVE_ACC_SHIFT            6
#define DA7280_E_LIM_DRIVE_ACC_MASK             (1 << 6)
#define DA7280_E_LIM_DRIVE_SHIFT                7
#define DA7280_E_LIM_DRIVE_MASK                 (1 << 7)

/* DA7280_IRQ_EVENT_PAT_DIAG (Address 0x0005) */
#define DA7280_E_PWM_FAULT_SHIFT                    5
#define DA7280_E_PWM_FAULT_MASK                     (1 << 5)
#define DA7280_E_MEM_FAULT_SHIFT                    6
#define DA7280_E_MEM_FAULT_MASK                     (1 << 6)
#define DA7280_E_SEQ_ID_FAULT_SHIFT                 7
#define DA7280_E_SEQ_ID_FAULT_MASK                  (1 << 7)

/* DA7280_IRQ_STATUS1 (Address 0x0006) */
#define DA7280_STA_SEQ_CONTINUE_SHIFT                      0
#define DA7280_STA_SEQ_CONTINUE_MASK                       (1 << 0)
#define DA7280_STA_UVLO_VBAT_OK_SHIFT                      1
#define DA7280_STA_UVLO_VBAT_OK_MASK                       (1 << 1)
#define DA7280_STA_PAT_DONE_SHIFT                          2
#define DA7280_STA_PAT_DONE_MASK                           (1 << 2)
#define DA7280_STA_OVERTEMP_CRIT_SHIFT                     3
#define DA7280_STA_OVERTEMP_CRIT_MASK                      (1 << 3)
#define DA7280_STA_PAT_FAULT_SHIFT                         4
#define DA7280_STA_PAT_FAULT_MASK                          (1 << 4)
#define DA7280_STA_WARNING_SHIFT                           5
#define DA7280_STA_WARNING_MASK                            (1 << 5)
#define DA7280_STA_ACTUATOR_SHIFT                          6
#define DA7280_STA_ACTUATOR_MASK                           (1 << 6)
#define DA7280_STA_OC_SHIFT                                7
#define DA7280_STA_OC_MASK                                 (1 << 7)

/* DA7280_IRQ_MASK1 (Address 0x0007) */
#define DA7280_SEQ_CONTINUE_M_SHIFT                          0
#define DA7280_SEQ_CONTINUE_M_MASK                           (1 << 0)
#define DA7280_UVLO_VBAT_OK_M_SHIFT                          1
#define DA7280_UVLO_VBAT_OK_M_MASK                           (1 << 1)
#define DA7280_PAT_DONE_M_SHIFT                              2
#define DA7280_PAT_DONE_M_MASK                               (1 << 2)
#define DA7280_OVERTEMP_CRIT_M_SHIFT                         3
#define DA7280_OVERTEMP_CRIT_M_MASK                          (1 << 3)
#define DA7280_PAT_FAULT_M_SHIFT                             4
#define DA7280_PAT_FAULT_M_MASK                              (1 << 4)
#define DA7280_WARNING_M_SHIFT                               5
#define DA7280_WARNING_M_MASK                                (1 << 5)
#define DA7280_ACTUATOR_M_SHIFT                              6
#define DA7280_ACTUATOR_M_MASK                               (1 << 6)
#define DA7280_OC_M_SHIFT                                    7
#define DA7280_OC_M_MASK                                     (1 << 7)

/* DA7280_CIF_I2C1 (Address 0x0008) */
#define DA7280_I2C_TO_ENABLE_SHIFT                            6
#define DA7280_I2C_TO_ENABLE_MASK                             (1 << 6)
#define DA7280_I2C_WR_MODE_SHIFT                              7
#define DA7280_I2C_WR_MODE_MASK                               (1 << 7)

/* DA7280_FRQ_LRA_PER_H (Address 0x000a) */
#define DA7280_LRA_PER_H_SHIFT                           0
#define DA7280_LRA_PER_H_MASK                            (255 << 0)

/* DA7280_FRQ_LRA_PER_L (Address 0x000b) */
#define DA7280_LRA_PER_L_SHIFT                           0
#define DA7280_LRA_PER_L_MASK                            (127 << 0)

/* DA7280_ACTUATOR1 (Address 0x000c) */
#define DA7280_ACTUATOR_NOMMAX_SHIFT                         0
#define DA7280_ACTUATOR_NOMMAX_MASK                          (255 << 0)

/* DA7280_ACTUATOR2 (Address 0x000d) */
#define DA7280_ACTUATOR_ABSMAX_SHIFT                         0
#define DA7280_ACTUATOR_ABSMAX_MASK                          (255 << 0)

/* DA7280_ACTUATOR3 (Address 0x000e) */
#define DA7280_IMAX_SHIFT                                    0
#define DA7280_IMAX_MASK                                     (31 << 0)

/* DA7280_CALIB_V2I_H (Address 0x000f) */
#define DA7280_V2I_FACTOR_H_SHIFT                          0
#define DA7280_V2I_FACTOR_H_MASK                           (255 << 0)

/* DA7280_CALIB_V2I_L (Address 0x0010) */
#define DA7280_V2I_FACTOR_L_SHIFT                          0
#define DA7280_V2I_FACTOR_L_MASK                           (255 << 0)

/* DA7280_CALIB_IMP_H (Address 0x0011) */
#define DA7280_IMPEDANCE_H_SHIFT                           0
#define DA7280_IMPEDANCE_H_MASK                            (255 << 0)

/* DA7280_CALIB_IMP_L (Address 0x0012) */
#define DA7280_IMPEDANCE_L_SHIFT                           0
#define DA7280_IMPEDANCE_L_MASK                            (3 << 0)

/* DA7280_TOP_CFG1 (Address 0x0013) */
#define DA7280_AMP_PID_EN_SHIFT                               0
#define DA7280_AMP_PID_EN_MASK                                (1 << 0)
#define DA7280_RAPID_STOP_EN_SHIFT                            1
#define DA7280_RAPID_STOP_EN_MASK                             (1 << 1)
#define DA7280_ACCELERATION_EN_SHIFT                          2
#define DA7280_ACCELERATION_EN_MASK                           (1 << 2)
#define DA7280_FREQ_TRACK_EN_SHIFT                            3
#define DA7280_FREQ_TRACK_EN_MASK                             (1 << 3)
#define DA7280_BEMF_SENSE_EN_SHIFT                           4
#define DA7280_BEMF_SENSE_EN_MASK                            (1 << 4)
#define DA7280_ACTUATOR_TYPE_SHIFT                            5
#define DA7280_ACTUATOR_TYPE_MASK                             (1 << 5)
#define DA7280_EMBEDDED_MODE_SHIFT                            7
#define DA7280_EMBEDDED_MODE_MASK                             (1 << 7)

/* DA7280_TOP_CFG2 (Address 0x0014) */
#define DA7280_FULL_BRAKE_THR_SHIFT                           0
#define DA7280_FULL_BRAKE_THR_MASK                            (15 << 0)
#define DA7280_MEM_DATA_SIGNED_SHIFT                          4
#define DA7280_MEM_DATA_SIGNED_MASK                           (1 << 4)

/* DA7280_TOP_CFG3 (Address 0x0015) */
#define DA7280_VBAT_MARGIN_SHIFT                              0
#define DA7280_VBAT_MARGIN_MASK                               (15 << 0)

/* DA7280_TOP_CFG4 (Address 0x0016) */
#define DA7280_TST_CALIB_IMPEDANCE_DIS_SHIFT                  6
#define DA7280_TST_CALIB_IMPEDANCE_DIS_MASK                   (1 << 6)
#define DA7280_V2I_FACTOR_FREEZE_SHIFT                        7
#define DA7280_V2I_FACTOR_FREEZE_MASK                         (1 << 7)

/* DA7280_TOP_INT_CFG1 (Address 0x0017) */
#define DA7280_BEMF_FAULT_LIM_SHIFT                       0
#define DA7280_BEMF_FAULT_LIM_MASK                        (3 << 0)
#define DA7280_FRQ_LOCKED_LIM_SHIFT                       2
#define DA7280_FRQ_LOCKED_LIM_MASK                        (63 << 2)

/* DA7280_TOP_INT_CFG6_H (Address 0x001c) */
#define DA7280_FRQ_PID_KP_H_SHIFT                       0
#define DA7280_FRQ_PID_KP_H_MASK                        (255 << 0)

/* DA7280_TOP_INT_CFG6_L (Address 0x001d) */
#define DA7280_FRQ_PID_KP_L_SHIFT                       0
#define DA7280_FRQ_PID_KP_L_MASK                        (255 << 0)

/* DA7280_TOP_INT_CFG7_H (Address 0x001e) */
#define DA7280_FRQ_PID_KI_H_SHIFT                       0
#define DA7280_FRQ_PID_KI_H_MASK                        (255 << 0)

/* DA7280_TOP_INT_CFG7_L (Address 0x001f) */
#define DA7280_FRQ_PID_KI_L_SHIFT                       0
#define DA7280_FRQ_PID_KI_L_MASK                        (255 << 0)

/* DA7280_TOP_INT_CFG8 (Address 0x0020) */
#define DA7280_TST_FRQ_TRACK_BEMF_LIM_SHIFT               0
#define DA7280_TST_FRQ_TRACK_BEMF_LIM_MASK                (15 << 0)
#define DA7280_TST_AMP_RAPID_STOP_LIM_SHIFT               4
#define DA7280_TST_AMP_RAPID_STOP_LIM_MASK                (7 << 4)

/* DA7280_TOP_CTL1 (Address 0x0022) */
#define DA7280_OPERATION_MODE_SHIFT                           0
#define DA7280_OPERATION_MODE_MASK                            (7 << 0)
#define DA7280_STANDBY_EN_SHIFT                    3
#define DA7280_STANDBY_EN_MASK                     (1 << 3)
#define DA7280_SEQ_START_SHIFT                                4
#define DA7280_SEQ_START_MASK                                 (1 << 4)

/* DA7280_TOP_CTL2 (Address 0x0023) */
#define DA7280_OVERRIDE_VAL_SHIFT                             0
#define DA7280_OVERRIDE_VAL_MASK                              (255 << 0)

/* DA7280_SEQ_CTL1 (Address 0x0024) */
#define DA7280_SEQ_CONTINUE_SHIFT                             0
#define DA7280_SEQ_CONTINUE_MASK                              (1 << 0)
#define DA7280_WAVEGEN_MODE_SHIFT                             1
#define DA7280_WAVEGEN_MODE_MASK                              (1 << 1)
#define DA7280_FREQ_WAVEFORM_TIMEBASE_SHIFT                   2
#define DA7280_FREQ_WAVEFORM_TIMEBASE_MASK                    (1 << 2)

/* DA7280_SWG_C1 (Address 0x0025) */
#define DA7280_CUSTOM_WAVE_GEN_COEFF1_SHIFT                     0
#define DA7280_CUSTOM_WAVE_GEN_COEFF1_MASK                      (255 << 0)

/* DA7280_SWG_C2 (Address 0x0026) */
#define DA7280_CUSTOM_WAVE_GEN_COEFF2_SHIFT                     0
#define DA7280_CUSTOM_WAVE_GEN_COEFF2_MASK                      (255 << 0)

/* DA7280_SWG_C3 (Address 0x0027) */
#define DA7280_CUSTOM_WAVE_GEN_COEFF3_SHIFT                     0
#define DA7280_CUSTOM_WAVE_GEN_COEFF3_MASK                      (255 << 0)

/* DA7280_SEQ_CTL2 (Address 0x0028) */
#define DA7280_PS_SEQ_ID_SHIFT                                0
#define DA7280_PS_SEQ_ID_MASK                                 (15 << 0)
#define DA7280_PS_SEQ_LOOP_SHIFT                              4
#define DA7280_PS_SEQ_LOOP_MASK                               (15 << 4)

/* DA7280_GPIO_0_CTL (Address 0x0029) */
#define DA7280_GPI0_POLARITY_SHIFT                         0
#define DA7280_GPI0_POLARITY_MASK                          (3 << 0)
#define DA7280_GPI0_MODE_SHIFT                             2
#define DA7280_GPI0_MODE_MASK                              (1 << 2)
#define DA7280_GPI0_SEQUENCE_ID_SHIFT                      3
#define DA7280_GPI0_SEQUENCE_ID_MASK                       (15 << 3)

/* DA7280_GPIO_1_CTL (Address 0x002a) */
#define DA7280_GPI1_POLARITY_SHIFT                         0
#define DA7280_GPI1_POLARITY_MASK                          (3 << 0)
#define DA7280_GPI1_MODE_SHIFT                             2
#define DA7280_GPI1_MODE_MASK                              (1 << 2)
#define DA7280_GPI1_SEQUENCE_ID_SHIFT                      3
#define DA7280_GPI1_SEQUENCE_ID_MASK                       (15 << 3)

/* DA7280_GPIO_2_CTL (Address 0x002b) */
#define DA7280_GPI2_POLARITY_SHIFT                         0
#define DA7280_GPI2_POLARITY_MASK                          (3 << 0)
#define DA7280_GPI2_MODE_SHIFT                             2
#define DA7280_GPI2_MODE_MASK                              (1 << 2)
#define DA7280_GPI2_SEQUENCE_ID_SHIFT                      3
#define DA7280_GPI2_SEQUENCE_ID_MASK                       (15 << 3)

/* DA7280_MEM_CTL1 (Address 0x002c) */
#define DA7280_PATTERN_BASE_ADDR_SHIFT                        0
#define DA7280_PATTERN_BASE_ADDR_MASK                         (255 << 0)

/* DA7280_MEM_CTL2 (Address 0x002d) */
#define DA7280_PATTERN_MEM_LOCK_SHIFT                         7
#define DA7280_PATTERN_MEM_LOCK_MASK                          (1 << 7)

/* DA7280_ADC_DATA_H1 (Address 0x002e) */
#define DA7280_ADC_VBAT_H_SHIFT                            0
#define DA7280_ADC_VBAT_H_MASK                             (255 << 0)

/* DA7280_ADC_DATA_L1 (Address 0x002f) */
#define DA7280_ADC_VBAT_L_SHIFT                            0
#define DA7280_ADC_VBAT_L_MASK                             (127 << 0)

/* DA7280_POLARITY (Address 0x0043) */
#define DA7280_POLARITY_SHIFT                                 0
#define DA7280_POLARITY_MASK                                  (1 << 0)

/* DA7280_LRA_AVR_H (Address 0x0044) */
#define DA7280_LRA_PER_AVERAGE_H_SHIFT                       0
#define DA7280_LRA_PER_AVERAGE_H_MASK                        (255 << 0)

/* DA7280_LRA_AVR_L (Address 0x0045) */
#define DA7280_LRA_PER_AVERAGE_L_SHIFT                       0
#define DA7280_LRA_PER_AVERAGE_L_MASK                        (127 << 0)

/* DA7280_FRQ_LRA_PER_ACT_H (Address 0x0046) */
#define DA7280_LRA_PER_ACTUAL_H_SHIFT                0
#define DA7280_LRA_PER_ACTUAL_H_MASK                 (255 << 0)

/* DA7280_FRQ_LRA_PER_ACT_L (Address 0x0047) */
#define DA7280_LRA_PER_ACTUAL_L_SHIFT                0
#define DA7280_LRA_PER_ACTUAL_L_MASK                 (127 << 0)

/* DA7280_FRQ_PHASE_H (Address 0x0048) */
#define DA7280_PHASE_SHIFT_H_SHIFT                         0
#define DA7280_PHASE_SHIFT_H_MASK                          (255 << 0)

/* DA7280_FRQ_PHASE_L (Address 0x0049) */
#define DA7280_PHASE_SHIFT_L_SHIFT                         0
#define DA7280_PHASE_SHIFT_L_MASK                          (7 << 0)
#define DA7280_PHASE_SHIFT_FREEZE_SHIFT                    7
#define DA7280_PHASE_SHIFT_FREEZE_MASK                     (1 << 7)

/* DA7280_FRQ_CTL (Address 0x004c) */
#define DA7280_FREQ_TRACKING_FORCE_ON_SHIFT                    0
#define DA7280_FREQ_TRACKING_FORCE_ON_MASK                     (1 << 0)
#define DA7280_FREQ_TRACKING_AUTO_ADJ_SHIFT                    1
#define DA7280_FREQ_TRACKING_AUTO_ADJ_MASK                     (1 << 1)

/* DA7280_TRIM3 (Address 0x005f) */
#define DA7280_REF_UVLO_THRES_SHIFT                              3
#define DA7280_REF_UVLO_THRES_MASK                               (3 << 3)
#define DA7280_LOOP_FILT_LOW_BW_SHIFT                            5
#define DA7280_LOOP_FILT_LOW_BW_MASK                             (1 << 5)
#define DA7280_LOOP_IDAC_DOUBLE_RANGE_SHIFT                      6
#define DA7280_LOOP_IDAC_DOUBLE_RANGE_MASK                       (1 << 6)

/* DA7280_TRIM4 (Address 0x0060) */
#define DA7280_LOOP_FILT_RES_TRIM_SHIFT                          0
#define DA7280_LOOP_FILT_RES_TRIM_MASK                           (3 << 0)
#define DA7280_LOOP_FILT_CAP_TRIM_SHIFT                          2
#define DA7280_LOOP_FILT_CAP_TRIM_MASK                           (3 << 2)

/* DA7280_TRIM6 (Address 0x0062) */
#define DA7280_HBRIDGE_ERC_HS_TRIM_SHIFT                         0
#define DA7280_HBRIDGE_ERC_HS_TRIM_MASK                          (3 << 0)
#define DA7280_HBRIDGE_ERC_LS_TRIM_SHIFT                         2
#define DA7280_HBRIDGE_ERC_LS_TRIM_MASK                          (3 << 2)

/* DA7280_IRQ_EVENT_ACTUATOR_FAULT (Address 0x0081) */
#define DA7280_E_TEST_ADC_SAT_FAULT_SHIFT     2
#define DA7280_E_TEST_ADC_SAT_FAULT_MASK      (1 << 2)

/* DA7280_IRQ_STATUS2 (Address 0x0082) */
#define DA7280_STA_TEST_ADC_SAT_SHIFT                      7
#define DA7280_STA_TEST_ADC_SAT_MASK                       (1 << 7)

/* DA7280_IRQ_MASK2 (Address 0x0083) */
#define DA7280_TEST_ADC_SAT_M_SHIFT                          7
#define DA7280_TEST_ADC_SAT_M_MASK                           (1 << 7)

/* DA7280_SNP_MEM_XX (Address 0x0084 ~ 0x00e7) */
#define DA7280_SNP_MEM_SHIFT                               0
#define DA7280_SNP_MEM_MASK                                (255 << 0)

#endif /* End of _DA7280_REG_DEFS_H */
