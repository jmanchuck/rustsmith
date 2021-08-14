use strum::IntoEnumIterator;

pub mod expr;
pub mod stmt;
pub mod type_weights;

trait EnumWeights: IntoEnumIterator {
    fn weights() -> Vec<u32> {
        Self::iter().map(|x| x.weight()).collect::<Vec<u32>>()
    }

    fn weight(&self) -> u32;
}
