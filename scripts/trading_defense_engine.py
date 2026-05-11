#!/usr/bin/env python3
"""
Trading Defense Engine (TDE)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Real-time adversarial probe detection for:
  - Ω KERNEL HFT (tick-to-trade latency monitoring)
  - Matriarch swarm (agent consensus coherence)
  - Firehorse spectral (order book correlation decay)
  - Singularity bot (cross-chain sync)

Monitors measurement strength γ in real-time.
When γ → γ_c: triggers automated defenses.

DEPLOY: pip install -r requirements.txt && python3 trading_defense_engine.py
"""

import asyncio
import json
import numpy as np
from datetime import datetime, timedelta
from dataclasses import dataclass, asdict
from typing import Dict, List, Optional
from enum import Enum
import logging
from prometheus_client import start_http_server, Gauge, Enum as PromEnum

# ============================================================================
# LOGGING SETUP
# ============================================================================

logging.basicConfig(
    level=logging.INFO,
    format='[%(asctime)s] [%(name)s] %(levelname)s: %(message)s'
)
logger = logging.getLogger("TDE")

# ============================================================================
# PROMETHEUS METRICS (DM-8 ALIGNED)
# ============================================================================

G_GAMMA = Gauge('tde_measurement_strength', 'Current measurement strength gamma (0-1)')
G_COHERENCE = Gauge('tde_coherence', 'System coherence [0, 1]')
G_PHASE = Gauge('tde_phase', 'Current system phase (0=PROTECTED, 1=CRITICAL, 2=LOCKDOWN)')
G_ACTIONS = Gauge('tde_defense_actions_triggered', 'Total defense actions triggered')
G_THREAT = Gauge('tde_threat_source_code', 'Categorized threat source ID')

# ============================================================================
# THREAT ENUMS
# ============================================================================

class DefenseMode(Enum):
    """System defense postures."""
    GREEN = "GREEN"          # Normal operation, γ < 0.3 * γ_c
    YELLOW = "YELLOW"        # Elevated probing, 0.3 * γ_c < γ < 0.7 * γ_c
    ORANGE = "ORANGE"        # High dissipation, 0.7 * γ_c < γ < γ_c
    RED = "RED"              # Critical threshold reached, γ ≈ γ_c
    LOCKDOWN = "LOCKDOWN"    # Full defense, γ > γ_c


class ThreatSource(Enum):
    """Where probing originates."""
    MEV_SEARCHER = "MEV_SEARCHER"           # Sandwich bot detecting trades
    FRONT_RUNNER = "FRONT_RUNNER"           # Direct front-running
    INFORMATION_LEAK = "INFORMATION_LEAK"   # Order leak via RPC/data provider
    MARKET_STRESS = "MARKET_STRESS"         # Natural volatility (false positive)
    UNKNOWN = "UNKNOWN"


# ============================================================================
# STATE TRACKING
# ============================================================================

@dataclass
class MeasurementSnapshot:
    """Single measurement of system dissipation."""
    timestamp: float
    gamma: float                 # Estimated measurement strength [0, 1]
    gamma_c: float              # Critical threshold
    phase: str                  # PROTECTED, CRITICAL, COLLAPSED
    entropy: float              # System entropy
    coherence: float            # [0, 1], 1 = fully coherent
    threat_source: ThreatSource
    confidence: float           # [0, 1], confidence in γ estimate


@dataclass
class DefenseAction:
    """Action to take in response to threat."""
    mode: DefenseMode
    timestamp: float
    gamma: float
    action: str                 # Description of what to do
    priority: int               # 1 = critical, 5 = informational
    executed: bool = False


# ============================================================================
# KERNEL PROBE DETECTOR (Ω KERNEL)
# ============================================================================

class OmegaKernelProbeDetector:
    """
    Monitor Ω KERNEL for latency signatures of front-running.
    
    Principle: If adversary is probing your orders, latency will increase
    as they scan nearby price levels. We detect this via tick-to-trade
    latency distribution anomalies.
    """
    
    def __init__(self, kernel_metrics_url="http://localhost:5555/metrics"):
        self.url = kernel_metrics_url
        self.latency_window = 100  # Keep last 100 trades
        self.latencies = []
        self.baseline_mean = 60.0  # ns, normal Ω KERNEL latency
        self.baseline_std = 2.0    # ns
    
    async def fetch_latest_latencies(self) -> List[float]:
        """
        Fetch tick-to-trade latencies from Ω KERNEL metrics.
        Returns: [latency_ns] for last N trades
        """
        # TODO: Implement HTTP call to your kernel metrics endpoint
        # For now, mock with realistic data
        
        # Normal case: ~60ns with ±2ns jitter
        if np.random.rand() > 0.1:  # 90% normal
            return list(np.random.normal(self.baseline_mean, self.baseline_std, 10))
        else:  # 10% under probe (latency spikes to 80-100ns)
            return list(np.random.normal(90, 5, 10))
    
    async def measure_gamma(self) -> float:
        """
        Estimate measurement strength γ from latency anomaly.
        
        γ ≈ (actual_latency - baseline_latency) / baseline_latency
        """
        latencies = await self.fetch_latest_latencies()
        self.latencies.extend(latencies)
        self.latencies = self.latencies[-self.latency_window:]
        
        mean_latency = float(np.mean(self.latencies))
        anomaly = max(0, mean_latency - self.baseline_mean)
        
        # γ = fractional increase in latency
        gamma = anomaly / self.baseline_mean if self.baseline_mean > 0 else 0.0
        
        return float(gamma)


# ============================================================================
# MATRIARCH COHERENCE DETECTOR (Swarm)
# ============================================================================

class MatriarchCoherenceDetector:
    """
    Monitor Matriarch swarm consensus strength.
    
    Principle: Measurement causes agent reputations to decorrelate.
    We track the entropy of the reputation distribution.
    """
    
    def __init__(self, matriarch_api="http://localhost:5000"):
        self.api = matriarch_api
        self.entropy_window = 50
        self.entropy_history = []
        self.baseline_entropy = 3.0  # nats, for 64-agent swarm
    
    async def fetch_agent_reputations(self) -> Dict[str, float]:
        """
        Fetch all agent reputations from Matriarch API.
        Returns: {agent_id: reputation}
        """
        # TODO: GET /api/agents/reputation
        # For now, mock
        return {f"agent_{i}": np.random.rand() for i in range(64)}
    
    async def measure_entropy(self) -> float:
        """Compute Shannon entropy of agent reputation distribution."""
        reps = await self.fetch_agent_reputations()
        values = np.array(list(reps.values())) + 1e-10
        probs = values / np.sum(values)
        entropy = float(-np.sum(probs * np.log(probs)))
        
        self.entropy_history.append(entropy)
        self.entropy_history = self.entropy_history[-self.entropy_window:]
        
        return entropy
    
    async def measure_gamma(self) -> float:
        """
        Estimate γ from entropy decay rate.
        
        Under dissipation: dS/dt = -k * γ * S
        So: γ ≈ (baseline - current) / (baseline * time_constant)
        """
        S_current = await self.measure_entropy()
        S_baseline = self.baseline_entropy
        
        if len(self.entropy_history) < 2:
            return 0.0
        
        # Rate of entropy decay
        dS_dt = (self.entropy_history[-2] - self.entropy_history[-1])
        
        # γ proportional to decay rate
        gamma = max(0, dS_dt / (S_baseline + 1e-6))
        
        return float(gamma)


# ============================================================================
# FIREHORSE CORRELATION DETECTOR (Spectral)
# ============================================================================

class FirehorseCorrelationDetector:
    """
    Monitor Firehorse spectral trading for order book correlation decay.
    
    Principle: Topological order = long-range correlations in order book flow.
    Front-running causes these to collapse (high dissipation).
    """
    
    def __init__(self, redis_url="redis://localhost:6379"):
        self.redis_url = redis_url
        self.correlation_window = 30
        self.correlation_history = []
        self.baseline_correlation = 0.8  # Strong long-range correlation
    
    async def fetch_order_book_correlations(self) -> float:
        """
        Compute autocorrelation of order flow at different lags.
        High correlation = strong topological order (buying/selling patterns coherent)
        Low correlation = decorrelated (measurement / front-running)
        
        Returns: average autocorrelation at lag=10
        """
        # TODO: Pull from Redis Firehorse cache
        # Simulate: normal = 0.8, under attack = 0.3-0.5
        if np.random.rand() > 0.15:
            return float(np.random.normal(0.8, 0.05))
        else:
            return float(np.random.normal(0.4, 0.1))
    
    async def measure_gamma(self) -> float:
        """
        Estimate γ from correlation loss.
        
        γ ≈ 1 - (actual_correlation / baseline_correlation)
        """
        corr = await self.fetch_order_book_correlations()
        self.correlation_history.append(corr)
        self.correlation_history = self.correlation_history[-self.correlation_window:]
        
        # Loss = how much correlation degraded
        loss = max(0, self.baseline_correlation - corr)
        gamma = loss / (self.baseline_correlation + 1e-6)
        
        return float(gamma)


# ============================================================================
# THREAT INFERENCE ENGINE
# ============================================================================

class ThreatInferenceEngine:
    """
    Fuse signals from Ω KERNEL, Matriarch, Firehorse to infer threat source.
    """
    
    async def infer_threat(
        self,
        gamma_kernel: float,
        gamma_swarm: float,
        gamma_spectral: float
    ) -> ThreatSource:
        """
        Determine likely threat source based on which systems are probed.
        
        Heuristics:
        - Kernel + Spectral high, Swarm low → MEV searcher (targeting prices, not agents)
        - All three high → Information leak (broad market probe)
        - Swarm high, others low → Agent corruptor (targeting decision-making)
        - Spectral only → Natural market stress
        """
        
        threshold = 0.3
        
        kernel_probed = gamma_kernel > threshold
        swarm_probed = gamma_swarm > threshold
        spectral_probed = gamma_spectral > threshold
        
        if kernel_probed and spectral_probed and not swarm_probed:
            return ThreatSource.MEV_SEARCHER
        elif kernel_probed and not spectral_probed:
            return ThreatSource.FRONT_RUNNER
        elif swarm_probed and kernel_probed and spectral_probed:
            return ThreatSource.INFORMATION_LEAK
        elif spectral_probed and not (kernel_probed or swarm_probed):
            return ThreatSource.MARKET_STRESS
        else:
            return ThreatSource.UNKNOWN


# ============================================================================
# DEFENSE RESPONSE ENGINE
# ============================================================================

class DefenseResponseEngine:
    """
    Generate automated responses based on threat level.
    """
    
    def __init__(self):
        self.actions_executed = []
    
    async def plan_defense(
        self,
        gamma: float,
        gamma_c: float,
        threat_source: ThreatSource
    ) -> List[DefenseAction]:
        """
        Generate ordered list of defense actions.
        """
        actions = []
        
        # Determine mode
        if gamma > gamma_c:
            mode = DefenseMode.LOCKDOWN
        elif gamma > 0.7 * gamma_c:
            mode = DefenseMode.RED
        elif gamma > 0.3 * gamma_c:
            mode = DefenseMode.ORANGE
        elif gamma > 0.1 * gamma_c:
            mode = DefenseMode.YELLOW
        else:
            mode = DefenseMode.GREEN
        
        timestamp = datetime.now().timestamp()
        
        # Generate actions based on mode and threat
        if mode == DefenseMode.LOCKDOWN:
            actions.append(DefenseAction(
                mode=mode,
                timestamp=timestamp,
                gamma=gamma,
                action="PAUSE_ALL_ORDERS - Full defensive lockdown",
                priority=1
            ))
            actions.append(DefenseAction(
                mode=mode,
                timestamp=timestamp,
                gamma=gamma,
                action="RECORD_VOID_MEMORY - Submit incident to blockchain",
                priority=1
            ))
        
        elif mode == DefenseMode.RED:
            actions.append(DefenseAction(
                mode=mode,
                timestamp=timestamp,
                gamma=gamma,
                action="REDUCE_ORDER_SIZE - Cut position by 50%",
                priority=1
            ))
            actions.append(DefenseAction(
                mode=mode,
                timestamp=timestamp,
                gamma=gamma,
                action="SWITCH_PRIVATE - Route orders through Jito private pool",
                priority=2
            ))
            
            if threat_source == ThreatSource.MEV_SEARCHER:
                actions.append(DefenseAction(
                    mode=mode,
                    timestamp=timestamp,
                    gamma=gamma,
                    action="ACTIVATE_MEV_BUNDLES - Submit via Jito bundles",
                    priority=2
                ))
        
        elif mode == DefenseMode.ORANGE:
            actions.append(DefenseAction(
                mode=mode,
                timestamp=timestamp,
                gamma=gamma,
                action="RANDOMIZE_ORDER_TIMING - Add latency jitter (±100ms)",
                priority=3
            ))
            actions.append(DefenseAction(
                mode=mode,
                timestamp=timestamp,
                gamma=gamma,
                action="SPLIT_ORDERS - Break large orders into 5-10 pieces",
                priority=3
            ))
        
        elif mode == DefenseMode.YELLOW:
            actions.append(DefenseAction(
                mode=mode,
                timestamp=timestamp,
                gamma=gamma,
                action="INCREASE_MONITORING - Log all trades to Void Memory",
                priority=4
            ))
        
        return actions
    
    async def execute_action(self, action: DefenseAction) -> bool:
        """
        Execute a defense action. Returns True if successful.
        """
        logger.info(f"[DEFENSE] [{action.mode.value}] {action.action}")
        
        # TODO: Integrate with actual systems
        # - Pause orders: HTTP call to Matriarch
        # - Switch private: Modify router config in Firehorse
        # - Bundle: Construct Jito bundle with pending orders
        # - Record: Submit to blockchain Void Memory
        
        action.executed = True
        self.actions_executed.append(action)
        return True


# ============================================================================
# MAIN TRADING DEFENSE ENGINE
# ============================================================================

class TradingDefenseEngine:
    """
    Unified real-time adversarial probe detector and auto-defense system.
    """
    
    def __init__(self):
        self.kernel_detector = OmegaKernelProbeDetector()
        self.swarm_detector = MatriarchCoherenceDetector()
        self.spectral_detector = FirehorseCorrelationDetector()
        self.threat_engine = ThreatInferenceEngine()
        self.defense_engine = DefenseResponseEngine()
        
        # Calibration
        self.gamma_c = 0.5  # Critical threshold (to be measured)
        self.update_interval = 1.0  # seconds
        
        self.snapshots: List[MeasurementSnapshot] = []
        self.actions: List[DefenseAction] = []
        self.is_running = False
    
    async def measure_system_state(self) -> MeasurementSnapshot:
        """
        Take a snapshot of all system measurements.
        Fuse into single γ estimate.
        """
        gamma_k = await self.kernel_detector.measure_gamma()
        gamma_s = await self.swarm_detector.measure_gamma()
        gamma_f = await self.spectral_detector.measure_gamma()
        
        # Fused estimate: weighted average
        gamma_fused = 0.4 * gamma_k + 0.3 * gamma_s + 0.3 * gamma_f
        
        # Infer threat
        threat = await self.threat_engine.infer_threat(gamma_k, gamma_s, gamma_f)
        
        # Determine phase
        if gamma_fused > self.gamma_c:
            phase = "COLLAPSED"
        elif gamma_fused > 0.7 * self.gamma_c:
            phase = "CRITICAL"
        else:
            phase = "PROTECTED"
        
        # Estimate entropy (proxy for coherence)
        entropy = await self.swarm_detector.measure_entropy()
        coherence = max(0, 1.0 - gamma_fused)
        
        snapshot = MeasurementSnapshot(
            timestamp=datetime.now().timestamp(),
            gamma=gamma_fused,
            gamma_c=self.gamma_c,
            phase=phase,
            entropy=entropy,
            coherence=coherence,
            threat_source=threat,
            confidence=0.7  # TODO: compute actual confidence
        )
        
        self.snapshots.append(snapshot)
        self.snapshots = self.snapshots[-1000:]  # Keep last 1000
        
        return snapshot
    
    async def run_defense_loop(self):
        """
        Main loop: measure → infer → defend.
        Runs continuously.
        """
        logger.info("="*70)
        logger.info("Trading Defense Engine STARTED")
        logger.info(f"Critical threshold γ_c = {self.gamma_c}")
        logger.info("="*70)
        
        self.is_running = True
        
        try:
            while self.is_running:
                # Measure
                snapshot = await self.measure_system_state()
                
                # Log
                logger.info(
                    f"STATE: γ={snapshot.gamma:.4f} "
                    f"(γ_c={snapshot.gamma_c:.2f}) "
                    f"| Phase: {snapshot.phase} "
                    f"| Threat: {snapshot.threat_source.value} "
                    f"| Coherence: {snapshot.coherence:.2%}"
                )
                
                # Update Prometheus Metrics (DM-8 Alignment)
                G_GAMMA.set(snapshot.gamma)
                G_COHERENCE.set(snapshot.coherence)
                
                # Phase Mapping: PROTECTED=0, CRITICAL=1, LOCKDOWN=2
                phase_id = 0
                if snapshot.phase == "CRITICAL": phase_id = 1
                elif snapshot.phase == "COLLAPSED": phase_id = 2
                G_PHASE.set(phase_id)
                
                G_ACTIONS.set(len(self.actions))
                
                # Threat Mapping
                threat_map = { "MEV_SEARCHER": 1, "FRONT_RUNNER": 2, "INFORMATION_LEAK": 3, "MARKET_STRESS": 4, "UNKNOWN": 5 }
                G_THREAT.set(threat_map.get(snapshot.threat_source.value, 5))
                
                # Infer & Defend
                actions = await self.defense_engine.plan_defense(
                    snapshot.gamma,
                    snapshot.gamma_c,
                    snapshot.threat_source
                )
                
                # Execute highest-priority actions
                for action in sorted(actions, key=lambda a: a.priority):
                    await self.defense_engine.execute_action(action)
                    self.actions.append(action)
                
                await asyncio.sleep(self.update_interval)
        
        except KeyboardInterrupt:
            logger.info("Stopping...")
            self.is_running = False
        
        except Exception as e:
            logger.error(f"ERROR: {e}")
            import traceback
            traceback.print_exc()
    
    def get_status_report(self) -> Dict:
        """Generate current status for dashboard."""
        if not self.snapshots:
            return {"status": "no_data"}
        
        latest = self.snapshots[-1]
        
        return {
            "timestamp": datetime.fromtimestamp(latest.timestamp).isoformat(),
            "gamma": latest.gamma,
            "gamma_c": latest.gamma_c,
            "phase": latest.phase,
            "coherence": latest.coherence,
            "threat": latest.threat_source.value,
            "margin_percent": max(0, (latest.gamma_c - latest.gamma) / latest.gamma_c * 100),
            "actions_executed": len(self.actions),
            "last_action": self.actions[-1].action if self.actions else None,
        }


# ============================================================================
# CLI & DEPLOYMENT
# ============================================================================

async def main():
    tde = TradingDefenseEngine()
    
    # Optional: calibrate γ_c first
    logger.info("Calibrating γ_c from historical data...")
    # TODO: Run aethernode_operations.py to measure real γ_c
    # For now, assume 0.5
    
    # Start Prometheus server
    start_http_server(8999)
    logger.info("📊 TDE Metrics live at http://localhost:8999/metrics")

    await tde.run_defense_loop()


if __name__ == "__main__":
    asyncio.run(main())
